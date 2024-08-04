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

use crate::api::market::MarketAPI;
use crate::model::order::Order;
use crate::model::position::Position;
use crate::model::tick::Tick;
use crate::schema::positions::dsl::*;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct Broker{
    api: Box<dyn MarketAPI>,
    pool: crate::storage::postgres::DbPool,
    cache_position: Cache<String, Vec<Position>>,
}


impl Broker {
    pub fn new(api: Box<dyn MarketAPI>, database_url: String) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool");
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(300)) // 5 minutes
            .build();
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


    async fn execute_order(&self, order: Order) -> Result<(Order)> {
        match self.api.order(order).await {
            Ok(order) => {

                Ok(order)
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use dotenvy::dotenv;
    use tokio;

    use super::*;

    #[tokio::test]
    async fn test_get_positions() {
        // Create a mock database pool
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let broker = Broker::new(Box::new(MarketAPI::new()), database_url);

        // First call should fetch from DB
        let positions1 = broker.get_positions().await.unwrap();

        // Second call should return cached result
        let positions2 = broker.get_positions().await.unwrap();

        assert_eq!(positions1, positions2);

        // Wait for cache to expire
        tokio::time::sleep(Duration::from_secs(301)).await;

        // This call should fetch from DB again
        let positions3 = broker.get_positions().await.unwrap();

        // Assuming the DB content hasn't changed
        assert_eq!(positions1, positions3);
    }
}