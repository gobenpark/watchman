use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

pub mod position;

#[derive(Insertable, Selectable, Queryable, Debug)]
#[diesel(table_name = crate::schema::positions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Position {
    pub id: Uuid,
    pub ticker: String,
    pub price: f64,
    pub amount: f64,
    pub strategy_id: String,
    pub created_at: chrono::NaiveDateTime,
}
