use std::fmt::Display;

use anyhow::Result;
use chrono::Duration;
use diesel::{Insertable, PgConnection};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, deadpool};
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::scoped_futures::ScopedFutureExt;
use log::info;
use moka::future::Cache;
use teloxide::payloads::SetStickerPositionInSetSetters;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio_util::sync::CancellationToken;
use tracing::error;
use crate::api::market::OrderResultType;

use crate::api::market::MarketAPI;
use crate::model::order::{Order, OrderAction, NewOrder, OrderType};
use crate::model::position::Position;
use crate::model::tick::Tick;
use crate::schema::orders::dsl::*;
use crate::schema::positions::dsl as positiondsl;


type DbPool = Pool<AsyncPgConnection>;

pub struct Broker{
    api: Box<dyn MarketAPI>,
    pool: DbPool,
    cache_position: Cache<String, Vec<Position>>,
}


impl Broker {
    pub fn new(api: Box<dyn MarketAPI>, database_url: String) -> Self {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config).build().expect("fail to create pool builder");
        let cache = Cache::new(10_000);
        Self {api,pool,cache_position: cache}
    }

    pub async fn transaction(&self,ctx: CancellationToken,target: Vec<&String>) -> Result<Receiver<Tick>> {
        let (tx,rx) = channel::<Tick>(100);
        let mut receiver = self.api.connect_websocket(ctx.clone()).await?;
        tokio::spawn(async move {
            loop {
                select! {
                    tk = receiver.recv() => {
                        if let Some(tk) = tk {
                            tx.send(tk).await.unwrap()
                        }
                    }
                    _ = ctx.cancelled() => {
                        break;
                    }
                }
            }
        });

        for i in target{
            self.api.subscribe(i).await?;
        }
        Ok(rx)
    }


    pub async fn process_order(&self, ctx: CancellationToken) -> Result<()> {
        let pool = self.pool.clone();
        let mut receiver = self.api.connect_websocket_order_transaction(ctx.clone()).await?;
        let tself = self.clone();

        tokio::spawn(async move {
            loop {
                select! {
                Some(v) = receiver.recv() => {
                    match v.result {
                        OrderResultType::Success => {
                            let oid = v.id.parse::<i32>().unwrap();
                            let conn = &mut pool.get().await.expect("Failed to get DB connection");
                            let result = diesel::update(orders.find(oid))
                                    .set(accepted.eq(true))
                                    .execute(conn)
                                    .await;
                            match result {
                                Ok(_) => {
                                    info!("Order accepted: {}", v.id);
                                }
                                Err(e) => {
                                    error!("Failed to update order: {}", e);
                                }
                            }
                        },
                            OrderResultType::Cancel => {
                                let oid = v.id.parse::<i32>().unwrap();
                                let conn = &mut pool.get().await.expect("Failed to get DB connection");
                                let result =  diesel::delete(orders.find(oid))
                                        .execute(conn)
                                        .await;
                                match result {
                                    Ok(_) => {
                                        info!("Order cancelled: {}", v.id);
                                    }
                                    Err(e) => {
                                        error!("Failed to delete order: {}", e);
                                    }
                                }
                            }
                        _ => {}
                    }
                }
                _ = ctx.cancelled() => {
                    info!("Order processing cancelled");
                    break;
                }
            }
            }
        });

        Ok(())
    }


    async fn update_position(&self,o: Order) -> Result<()>{
        use crate::schema::positions::dsl::*;
        let conn = &mut self.pool.get().await.expect("Failed to get DB connection");
        let mut po = positions
            .filter(ticker.eq(o.ticker()).and(strategy_id.eq(o.strategy_id())))
            .select(Position::as_select())
            .first(conn).await?;
        let total = po.quantity() + o.quantity as f64;
        let avg_price = (po.price() * po.quantity() + o.price * o.quantity as f64) / total;
        diesel::update(positions).set((
            quantity.eq(total),
            price.eq(avg_price)
        )).execute(conn).await?;
        let cache_key = "all_positions".to_string();
        self.cache_position.remove(&cache_key).await;
        Ok(())

    }

    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        let cache_key = "all_positions".to_string();
        if let Some(cached_positions) = self.cache_position.get(&cache_key).await {
            return Ok(cached_positions);
        }
        let po = self.fetch_positions_from_db().await?;
        self.cache_position.insert(cache_key, po.clone()).await;
        Ok(po)
    }

    async fn fetch_positions_from_db(&self) -> Result<Vec<Position>> {
        let conn = &mut self.pool.get().await?;
        let results = positiondsl::positions.select(Position::as_select()).load(conn).await?;
        Ok(results)
    }


    pub async fn execute_order(&self, o: Order) -> Result<Order> {
        let conn = &mut self.pool.get().await?;
        let result = conn.transaction::<_,diesel::result::Error,_>(|conn| async move {
            match self.api.order(o).await {
                Ok(order) => {
                    info!("Order executed: {:?}", order);
                    let o1   = order.clone();
                    let od = NewOrder::from(o1);
                    diesel::insert_into(orders).values(od).execute(conn).await?;
                    Ok(order)
                }
                Err(e) => {
                    error!("Failed to execute order: {}", e);
                    Err(diesel::result::Error::RollbackTransaction)
                }
            }
        }.scope_boxed()).await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use dotenvy::dotenv;
    use tokio;
    use tokio::time::{sleep,Duration};
    use crate::api::lssec::LsSecClient;

    use super::*;

    #[tokio::test]
    async fn test_order() {
        // Create a mock database pool
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let broker = Broker::new(Box::new(LsSecClient::new(env::var("LSSEC_KEY").unwrap(),env::var("LSSEC_SECRET").unwrap())), database_url);

        let tk = CancellationToken::new();
        broker.process_order(tk).await;

        let od = Order::new(0,"432430".to_string(), 1, 57000.0,"test".to_string(), OrderAction::Buy, OrderType::Limit);

        broker.execute_order(
            od
        ).await.unwrap();
    }
    #[tokio::test]
    async fn test_get_positions() {
        // Create a mock database pool
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let broker = Broker::new(Box::new(LsSecClient::new(env::var("LSSEC_KEY").unwrap(), env::var("LSSEC_SECRET").unwrap())), database_url);


        let od = Order::new(0, "83923".to_string(), 1, 57000.0, "sampl2e".to_string(), OrderAction::Buy, OrderType::Limit);
        broker.update_position(od).await.unwrap();
    }

}