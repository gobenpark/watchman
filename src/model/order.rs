use std::fmt::Display;
use diesel::prelude::*;
use sea_orm::entity::prelude::*;
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
    pub fn as_str(&self) -> &str {
        match self {
            OrderAction::Sell => "1",
            OrderAction::Buy => "2",
        }
    }
}

impl OrderType {
    pub fn as_str(&self) -> &str {
        match self {
            OrderType::Limit => "00",
            OrderType::Market => "03",
        }
    }
}

#[derive(Insertable,Debug)]
#[diesel(table_name = crate::schema::orders)]
pub struct NewOrder {
    pub id: i32,
    pub ticker: String,
    pub quantity: i32,
    pub price: f64,
    pub strategy_id: String,
    #[diesel(column_name = "order_action")]
    pub action: String,
    accepted: bool,
    pub created_at: chrono::NaiveDateTime,
}


impl NewOrder {
    pub fn from(order: Order) -> Self {
        Self {
            id: order.id,
            ticker: order.ticker,
            quantity: order.quantity,
            price: order.price,
            strategy_id: "default".to_string(),
            action: {
                match order.action {
                    OrderAction::Buy => "2".to_string(),
                    OrderAction::Sell => "1".to_string(),
                }
            },
            accepted: false,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}



#[derive(Clone, Debug)]
pub struct Order {
    pub id: i32,
    ticker: String,
    pub quantity: i32,
    pub price: f64,
    pub strategy_id: String,
    pub action: OrderAction,
    pub order_type: OrderType,
}


impl Order {
    pub fn new(
        id: i32,
        ticker: String,
        quantity: i32,
        price: f64,
        strategy_id: String,
        action: OrderAction,
        order_type: OrderType,
    ) -> Self {
        Self {
            id,
            ticker,
            quantity,
            price,
            strategy_id,
            action,
            order_type,
        }
    }

    pub fn ticker(&self) -> &str {
        &self.ticker
    }

    pub fn strategy_id(&self) -> &str {
        &self.strategy_id
    }

    pub fn set_id(&mut self, id: i32) {
        self.id = id;
    }
}
