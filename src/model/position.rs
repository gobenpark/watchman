use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::SelectableHelper;

#[derive(Serialize, Deserialize, Clone)]
#[derive(Selectable, Queryable,AsChangeset, Debug)]
#[diesel(table_name = crate::schema::positions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Position {
    pub id: uuid::Uuid,
    pub ticker: String,
    pub quantity: f64,
    pub price: f64,
    pub strategy_id: String,
    pub created_at: chrono::NaiveDateTime,
}


#[derive(Insertable)]
#[diesel(table_name = crate::schema::positions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPosition {
    pub ticker: String,
    pub price: f64,
    pub quantity: f64,
    pub strategy_id: String,
}

