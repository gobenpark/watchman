use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::*;
use serde::{Deserialize, Serialize, Serializer};

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_order() {
        let order = Order::new(
            "1".to_string(),
            Some(1.0),
            Some(1.0),
            Utc::now().naive_utc(),
        );
        assert_eq!(order.id, "1");
        assert_eq!(order.price, Some(1.0));
        assert_eq!(order.amount, Some(1.0));
    }
}
