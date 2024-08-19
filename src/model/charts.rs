use diesel::{Queryable, Selectable};

#[derive(Clone, Debug)]
#[derive(Queryable,Selectable)]
#[diesel(table_name = crate::schema::charts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Charts {
    pub ticker: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i32,
    pub datetime: chrono::NaiveDateTime
}
