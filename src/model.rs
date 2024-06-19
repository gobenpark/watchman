use serde::{Deserialize, Serialize, Serializer};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_types::*;


#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub id: String,
    pub price: Option<f64>,
    pub amount: Option<f64>,
    pub created_at: NaiveDateTime,
}

impl Order {
    fn new(id: String, price: Option<f64>, amount: Option<f64>, created_at: NaiveDateTime) -> Self {
        Self {
            id,
            price,
            amount,
            created_at,
        }
    }
}
