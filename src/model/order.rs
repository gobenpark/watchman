use diesel::prelude::*;

#[derive(Copy, Clone, Debug)]
pub enum OrderAction {
    Buy,
    Sell,
}

#[derive(Copy, Clone, Debug)]
pub enum OrderType {
    Limit,
    Market,
}

impl OrderAction {
    fn as_str(&self) -> &str {
        match self {
            OrderAction::Sell => "1",
            OrderAction::Buy => "2",
        }
    }
}

impl OrderType {
    fn as_str(&self) -> &str {
        match self {
            OrderType::Limit => "00",
            OrderType::Market => "03",
        }
    }
}



#[derive(Clone, Debug)]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    id: i32,
    ticker: String,
    quantity: i64,
    price: i64,
    #[diesel(embed)]
    action: OrderAction,
    #[diesel(embed)]
    order_type: OrderType,
    pub created_at: chrono::NaiveDateTime,
}

impl Order {
    pub fn new(
        id: i64,
        ticker: String,
        quantity: i64,
        price: i64,
        action: OrderAction,
        order_type: OrderType,
    ) -> Self {
        Self {
            id,
            ticker,
            quantity,
            price,
            action,
            order_type,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
