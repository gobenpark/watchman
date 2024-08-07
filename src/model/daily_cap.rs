use diesel::{Queryable, Selectable};

#[derive(Clone, Debug)]
#[derive(Queryable,Selectable)]
#[diesel(table_name = crate::schema::daily_cap)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DailyCap {
    pub ticker: String,
    pub market_cap: Option<i64>,
    pub trading_volume: Option<i64>,
    pub change: Option<bigdecimal::BigDecimal>,
    pub datetime: chrono::NaiveDateTime
}
