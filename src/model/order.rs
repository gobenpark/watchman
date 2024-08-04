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

#[derive(Insertable)]
#[diesel(table_name = crate::schema::orders)]
pub struct OrderInserter {
    pub ticker: String,
    pub quantity: i32,
    pub price: f64,
    #[diesel(column_name = "order_action")]
    pub action: String,
    pub created_at: chrono::NaiveDateTime,
}


impl OrderInserter {
    pub fn from(order: Order) -> Self {
        Self {
            ticker: order.ticker,
            quantity: order.quantity,
            price: order.price,
            action: {
                match order.action {
                    OrderAction::Buy => "2".to_string(),
                    OrderAction::Sell => "1".to_string(),
                }
            },
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}



#[derive(Clone, Debug)]
pub struct Order {
    id: i32,
    ticker: String,
    quantity: i32,
    price: f64,

    action: OrderAction,
    order_type: OrderType,
}

impl Order {
    pub fn new(
        id: i32,
        ticker: String,
        quantity: i32,
        price: f64,
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
