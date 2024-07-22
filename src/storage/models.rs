use diesel::data_types::PgTimestamp;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub id: String,
    pub price: Option<f64>,
    pub amount: Option<f64>,
    pub created_at: PgTimestamp,
}
