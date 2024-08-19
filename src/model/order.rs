use diesel::{AsExpression, deserialize, FromSqlRow, serialize};
use diesel::deserialize::FromSql;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Integer;

#[derive(Copy, Clone, Debug,FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum OrderAction {
    Buy = 1,
    Sell = 2,
}


#[derive(Copy, Clone, Debug,FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum OrderType {
    Limit = 1,
    Market = 2,
}
impl ToSql<Integer, Pg> for OrderType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            OrderType::Limit => <i32 as ToSql<Integer,Pg>>::to_sql(&1, out),
            OrderType::Market => <i32 as ToSql<Integer,Pg>>::to_sql(&2, out)
        }
    }
}

impl FromSql<Integer, Pg> for OrderType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match <i32 as FromSql<Integer, Pg>>::from_sql(bytes)? {
            1 => Ok(OrderType::Limit),
            2 => Ok(OrderType::Market),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl ToSql<Integer, Pg> for OrderAction {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            OrderAction::Buy => <i32 as ToSql<Integer,Pg>>::to_sql(&1, out),
            OrderAction::Sell => <i32 as ToSql<Integer,Pg>>::to_sql(&2, out)
        }
    }
}

impl FromSql<Integer, Pg> for OrderAction {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match <i32 as FromSql<Integer, Pg>>::from_sql(bytes)? {
            1 => Ok(OrderAction::Buy),
            2 => Ok(OrderAction::Sell),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
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

#[derive(Insertable,Debug,Clone)]
#[diesel(table_name = crate::schema::orders)]
pub struct NewOrder {
    pub id: i32,
    pub ticker: String,
    pub quantity: i32,
    pub price: f64,
    pub strategy_id: String,
    #[diesel(column_name = "order_action")]
    pub action: OrderAction,
    pub order_type: OrderType
}


impl NewOrder {
    pub fn from(order: Order) -> Self {
        Self {
            id: order.id,
            ticker: order.ticker,
            quantity: order.quantity,
            price: order.price,
            strategy_id: "default".to_string(),
            action: order.action,
            order_type: order.order_type,
        }
    }
}



#[derive(Clone, Debug)]
#[derive(Queryable,Selectable)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub id: i32,
    ticker: String,
    pub quantity: i32,
    pub price: f64,
    pub strategy_id: String,
    #[diesel(column_name = "order_action")]
    pub action: OrderAction,
    pub order_type: OrderType
}


impl Order {
    pub fn new(
        id: i32,
        ticker: String,
        quantity: i32,
        price: f64,
        strategy_id: String,
        action: OrderAction,
    ) -> Self {
        Self {
            id,
            ticker,
            quantity,
            price,
            strategy_id,
            action,
            order_type: OrderType::Limit,
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
