use std::fmt::Display;

use anyhow::Result;
use chrono::Duration;
use diesel::{Insertable, PgConnection};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use moka::future::Cache;
use teloxide::payloads::SetStickerPositionInSetSetters;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio_util::sync::CancellationToken;
use tracing::error;

use crate::api::market::MarketAPI;
use crate::model::order::{Order,OrderInserter,OrderAction,OrderType};
use crate::model::position::Position;
use crate::model::tick::Tick;
use crate::schema::positions::dsl::*;
use crate::schema::orders::dsl::*;
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct Broker{
    api: Box<dyn MarketAPI>,
    pool: DbPool,
    cache_position: Cache<String, Vec<Position>>,
}


impl Broker {
    pub fn new(api: Box<dyn MarketAPI>, database_url: String) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool");
        let cache = Cache::new(10_000);
        Self {api,pool,cache_position: cache}
    }

    pub async fn transaction(&self,ctx: CancellationToken) -> Result<Receiver<Tick>> {
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
        Ok(rx)
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let cache_key = "all_positions".to_string();
        if let Some(cached_positions) = self.cache_position.get(&cache_key).await {
            return Ok(cached_positions);
        }
        let po = self.fetch_positions_from_db().await?;
        self.cache_position.insert(cache_key, po.clone()).await;
        Ok(po)
    }

    async fn fetch_positions_from_db(&self) -> Result<Vec<Position>> {
        let conn = &mut self.pool.get()?;
        let results = positions.select(Position::as_select()).load(conn)?;
        Ok(results)
    }


    async fn execute_order(&self, o: Order) -> Result<(Order)> {
        let o1 = o.clone();
        let conn = &mut self.pool.get()?;

        let result = match self.api.order(o1).await {
            Ok(order) => {
                println!("Order executed: {:?}", order);
                let o1   = order.clone();
                let od = OrderInserter::from(o1);
                diesel::insert_into(orders).values(od).execute(conn)?;
                Ok(order)
            }
            Err(e) => {
                println!("Failed to execute order: {}", e);
                error!("Failed to execute order: {}", e);
                Err(diesel::result::Error::RollbackTransaction)
            }
        }?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use diesel_async::RunQueryDsl;
    use dotenvy::dotenv;
    use tokio;
    use crate::api::lssec::LsSecClient;

    use super::*;

    #[tokio::test]
    async fn test_get_positions() {
        // Create a mock database pool
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let broker = Broker::new(Box::new(LsSecClient::new(env::var("LSSEC_KEY").unwrap(),env::var("LSSEC_SECRET").unwrap())), database_url);

        let od = Order::new(0,"005935".to_string(), 1, 57000.0, OrderAction::Buy, OrderType::Limit);

        broker.execute_order(
            od
        ).await.unwrap();

    }
}