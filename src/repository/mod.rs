use anyhow::{Context, Result};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use futures_util::future::lazy;
use moka::future::Cache;
use tokio::sync::OnceCell;
use crate::schema::orders::dsl::orders;
use crate::model::prelude::*;
use polars::prelude::*;
use polars::series::Series;
use serde::de::Unexpected::Char;

type DbPool = Pool<AsyncPgConnection>;

pub struct Repository {
    pool: DbPool,
    cache_position: OnceCell<Cache<String, Position>>,
}

impl Repository {
    pub fn new(database_url: String) -> Self {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config).build().expect("fail to create pool builder");
        Self { pool , cache_position: OnceCell::new()}
    }

    fn cache_key(ticker: &str, strategy_id: &str) -> String {
        format!("{}:{}", ticker, strategy_id)
    }

    async fn init_cache(&self) -> Result<&Cache<String,Position>> {
        let cache = self.cache_position.get_or_init(|| async {
            let cache = Cache::new(10_000);
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

    pub async fn get_position(&self,tk: String, stg: String) -> Result<Option<Position>> {
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

    pub async fn get_ohlcv(&self) -> Result<DataFrame> {
        use crate::schema::charts::dsl::*;
        let conn = &mut self.pool.get().await?;
        let dc = charts.select(Charts::as_select()).load(conn).await?;

        let df = DataFrame::new(vec![
           Series::new("ticker",dc.iter().map(|x| x.ticker.clone()).collect::<Vec<String>>()),
           Series::new("open",dc.iter().map(|x| x.open.clone()).collect::<Vec<f64>>()),
           Series::new("close",dc.iter().map(|x| x.close.clone()).collect::<Vec<f64>>()),
           Series::new("high",dc.iter().map(|x| x.high.clone()).collect::<Vec<f64>>()),
           Series::new("low",dc.iter().map(|x| x.low.clone()).collect::<Vec<f64>>()),
           Series::new("volume",dc.iter().map(|x| x.volume.clone()).collect::<Vec<i32>>()),
           Series::new("datetime",dc.iter().map(|x| x.datetime.clone()).collect::<Vec<chrono::NaiveDateTime>>()),
        ])?;

        // col("close").rolling_mean(RollingOptionsFixedWindow{
        //     window_size: 20,
        //     min_periods: 1,
        //     weights: None,
        //     center: false,
        //     fn_params: None,
        // }).alias("rolling_mean").clone()
        let result = df.clone().lazy().filter(col("ticker").eq(lit("005930"))).select([
            col("ticker"),
            col("close").rolling_mean(RollingOptionsFixedWindow{
                window_size: 20,
                min_periods: 20,
                weights: None,
                center: false,
                fn_params: None,
            }),
            col("datetime")
        ]).collect();

        println!("{:?}",result);
        // let result = df.select([
        //     (col("close").rolling_mean(RollingOptionsFixedWindow{
        //         window_size: 20,
        //         min_periods: 1,
        //         weights: None,
        //         center: false,
        //         fn_params: None,
        //     }).alias("rolling_mean"))
        // ])?;




        // tdf.with_column(ma20.rename("ma20"))?;

        // // 20일 이동평균 계산
        // let ma20 = df.select("close")?
        //     .to_series()
        //     .cast(&DataType::Float64)?
        //     .rolling_mean(20, RollingOptions {
        //         window_size: WindowSize::Fixed(20),
        //         min_periods: 1,
        //         center: false,
        //         weights: None,
        //     })?;
        //
        // // 이동평균 열 추가
        // df.with_column(ma20.rename("ma20"))?;

        Ok(df)
    }
}

#[cfg(test)]
mod test {
    use diesel_async::RunQueryDsl;
    use crate::model::order::{OrderAction, OrderType};
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
        let posi = repo.add_position(po).await.unwrap();
        println!("{:?}",posi);
    }
    #[tokio::test]
    async fn test_accept_order() {
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string());
        let data = repo.accept_order(2).await.unwrap();
        println!("{:?}",data);
    }

    #[tokio::test]
    async fn test_cap() {
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string());
        let data = repo.get_ohlcv().await.unwrap();
        println!("{:?}",data);
    }


}