use crate::model::position::Position;
use crate::schema::positions;
use crate::schema::positions::dsl::*;
use anyhow::Result;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{Insertable, PgConnection};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub struct PostgresStorage {
    pool: DbPool,
}

impl PostgresStorage {
    pub fn new(url: String) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool");
        Self { pool }
    }

    pub fn add_position(&self, position: Position) -> Result<()> {
        let con = &mut self.pool.get()?;
        diesel::insert_into(positions::table)
            .values(&position)
            .execute(con)?;
        Ok(())
    }

    pub fn get_position(&self, symbol: String) -> Result<Vec<Position>> {
        let con = &mut self.pool.get()?;
        let po = positions
            .select(Position::as_select())
            .filter(ticker.eq(symbol))
            .load(con)?;
        Ok(po)
    }

    pub fn get_positions(&self) -> Result<Vec<Position>> {
        let con = &mut self.pool.get()?;
        let po = positions.select(Position::as_select()).load(con)?;
        Ok(po)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::order::Order;
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
    use dotenvy::dotenv;
    use std::env;
    pub type DbPool = Pool<ConnectionManager<PgConnection>>;

    #[test]
    fn test_add_position() {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let po = PostgresStorage::new(database_url);

        po.add_position(Position::default())
        .expect("Failed to add position");
    }

    #[test]
    fn test_get_position() {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let po = PostgresStorage::new(database_url);
        let result = po.get_position("AAPL".to_string()).expect("");
        println!("{:?}", result);
    }

    #[test]
    fn test_get_positions() {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let po = PostgresStorage::new(database_url);
        let result = po.get_positions().expect("");
        println!("{:?}", result);
    }
}
