use anyhow::{Context, Result};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel::dsl::{exists, select};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use moka::future::Cache;
use polars::prelude::*;
use tokio::sync::OnceCell;

use crate::model::prelude::*;


type DbPool = Pool<AsyncPgConnection>;

const CACHE_SIZE: u64 = 10_000;

pub struct Repository {
    pool: DbPool,
    cache_position: OnceCell<Cache<(String, String), Position>>,
}

impl Repository {
    pub fn new(database_url: String) -> Result<Self> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config).build().context("fail to create pool builder")?;
        Ok(Self { pool , cache_position: OnceCell::new()})
    }

    fn cache_key(ticker: &str, strategy_id: &str) -> (String,String) {
        (ticker.to_string(), strategy_id.to_string())
    }

    async fn init_cache(&self) -> Result<&Cache<(String,String),Position>> {
        let cache = self.cache_position.get_or_init(|| async {
            let cache = Cache::new(CACHE_SIZE);
            use crate::schema::positions::dsl::*;
            let conn = &mut self.pool.get().await.expect("Failed to get DB connection");
            let all_positions: Vec<Position> = positions
                .select(Position::as_select())
                .load(conn)
                .await
                .expect("Failed to load positions");

            for position in all_positions {
                let key = Self::cache_key(&position.ticker, &position.strategy_id);
                cache.insert(key, position).await;
            }
            cache
        }).await;
        Ok(cache)
    }

    pub async fn get_position(&self,tk: &str, stg: String) -> Result<Option<Position>> {
        let cache = self.init_cache().await?;
        let cache_key = Self::cache_key(&tk, &stg);
        if let Some(position) = cache.get(&cache_key).await {
            return Ok(Some(position));
        }
        Ok(None)
    }

    pub async fn update_position(&self, po: Position) -> Result<Position> {
        use crate::schema::positions::dsl::*;
        let conn = &mut self.pool.get().await?;
        let po = diesel::update(positions.filter(id.eq(po.id)))
            .set(&po)
            .returning(Position::as_returning())
            .get_result(conn).await?;
        let cache = self.init_cache().await?;
        let cache_key = Self::cache_key(&po.ticker, &po.strategy_id);
        cache.insert(cache_key, po.clone()).await;
        Ok(po)
    }

    pub async fn add_position(&self,position: NewPosition) -> Result<Position>{
        use crate::schema::positions::dsl::*;
        let conn = &mut self.pool.get().await?;
        let po = diesel::insert_into(positions)
            .values(&position)
            .returning(Position::as_returning())
            .get_result(conn).await?;
        Ok(po)
    }

    pub async fn add_order(&self,order: NewOrder) -> Result<()> {
        use crate::schema::orders::dsl::orders;
        let conn = &mut self.pool.get().await?;
        diesel::insert_into(orders)
            .values(&order)
            .execute(conn).await?;
        Ok(())
    }

    pub async fn order(&self,oid: i32) -> Result<Order> {
        use crate::schema::orders::dsl::*;
        let conn = &mut self.pool.get().await?;
        let od = orders.select(Order::as_select()).filter(id.eq(oid)).first::<Order>(conn).await?;
        Ok(od)
    }

    pub async fn accept_order(&self,oid: i32) -> Result<Order> {
        use crate::schema::orders::dsl::*;
        let conn = &mut self.pool.get().await?;
        let od = diesel::update(orders.filter(id.eq(oid)))
            .set(accepted.eq(true))
            .returning(Order::as_returning())
            .get_result(conn).await?;
        Ok(od)
    }

    pub async fn delete_order(&self,oid: i32) -> Result<()> {
        use crate::schema::orders::dsl::*;
        let conn = &mut self.pool.get().await?;
        diesel::delete(orders.filter(id.eq(oid)))
            .execute(conn).await?;
        Ok(())
    }

    pub async fn get_daily_caps(&self) -> Result<Vec<DailyCap>> {
        use crate::schema::daily_cap::dsl::*;
        let conn = &mut self.pool.get().await?;
        let dc = daily_cap.select(DailyCap::as_select()).load(conn).await?;
        Ok(dc)
    }

    pub async fn get_ohlcv(&self,tk: &str) -> Result<Vec<Charts>> {
        use crate::schema::charts::dsl::*;
        let conn = &mut self.pool.get().await?;
        let query = charts.select(Charts::as_select());
        let dc: Vec<Charts> = if tk.is_empty(){
            query.load(conn).await?
        }else {
            query.filter(ticker.eq(tk)).load(conn).await?
        };
        Ok(dc)
    }

    pub async fn has_pending_order(&self,tk: &str) -> Result<bool>{
        use crate::schema::orders::dsl::*;
        use crate::schema::orders;
        let conn = &mut self.pool.get().await?;
        let result = select(exists(
            orders::table
                .filter(ticker.eq(tk)
                    .and(accepted.eq(false)))
        ))
            .get_result::<bool>(conn)
            .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_crud_order() {
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string());

        let po = NewPosition{
            ticker: "005930".to_string(),
            price: 1.0,
            quantity: 1.0,
            strategy_id: "sample".to_string(),
        };
    }
    #[tokio::test]
    async fn test_accept_order() -> Result<()>{
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string())?;
        let result = repo.has_pending_order("267260").await?;
        println!("{}",result);
        Ok(())
    }



}