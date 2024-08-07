use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;
use crate::schema::orders::dsl::orders;
use crate::model::prelude::*;

type DbPool = Pool<AsyncPgConnection>;

pub struct Repository {
    pool: DbPool,
}

impl Repository {
    pub fn new(database_url: String) -> Self {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder(config).build().expect("fail to create pool builder");
        Self { pool }
    }

    pub async fn get_position(&self,ticker: String, strategy_id: String) -> Result<Position> {
        use crate::schema::positions::dsl::*;
        let conn = &mut self.pool.get().await?;
        let po = positions.select(Position::as_select()).filter(ticker.eq(ticker)).filter(strategy_id.eq(strategy_id)).first::<Position>(conn).await?;
        Ok(po)
    }

    pub async fn update_position(&self, po: Position) -> Result<Position> {
        use crate::schema::positions::dsl::*;
        let conn = &mut self.pool.get().await?;
        let po = diesel::update(positions.filter(id.eq(po.id)))
            .set(&po)
            .returning(Position::as_returning())
            .get_result(conn).await?;
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
    async fn test_order() {
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string());
        repo.add_order(NewOrder {
            id: 2,
            ticker: "005930".to_string(),
            quantity: 1,
            price: 1.0,
            strategy_id: "sample".to_string(),
            action: OrderAction::Buy,
            order_type: OrderType::Limit,
        }).await.expect("error");
    }

    #[tokio::test]
    async fn test_accept_order() {
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string());
        let data = repo.accept_order(2).await.unwrap();
        println!("{:?}",data);
    }

    #[tokio::test]
    async fn test_get_position(){
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string());
        let mut data = repo.get_position("005930".to_string(),"sample".to_string()).await.unwrap();
        data.ticker = "005935".to_string();
        let data = repo.update_position(data).await;
        println!("{:?}",data);
    }

    #[tokio::test]
    async fn test_get_dailycap(){
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string());
        let data = repo.get_daily_caps().await.unwrap();
        println!("{:?}",data);
    }


}