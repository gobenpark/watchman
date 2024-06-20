#[cfg(test)]
mod test {
    use super::*;
    use crate::schema::orders::dsl::*;
    use crate::storage::models::Order;
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
    use dotenvy::dotenv;
    use std::env;
    pub type DbPool = Pool<ConnectionManager<PgConnection>>;

    #[test]
    fn test() {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool");

        let con = &mut pool.get().expect("Failed to get connection from pool");

        let results = orders
            .limit(12)
            .select(Order::as_select())
            .load(con)
            .expect("Error loading orders");

        for order in results {
            println!("{:?}", order.created_at);
        }
    }
}
