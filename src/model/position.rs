use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
#[derive(Serialize, Deserialize, Clone)]
#[derive(Insertable, Selectable, Queryable, Debug)]
#[diesel(table_name = crate::schema::positions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Position {
    pub id: uuid::Uuid,
    pub ticker: String,
    pub quantity: f64,
    pub
    price: f64,
    strategy_id: String,
    created_at: chrono::NaiveDateTime,
}