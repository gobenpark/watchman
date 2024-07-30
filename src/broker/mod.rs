pub mod lssec;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::Receiver;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Market {
    KOSPI,
    KOSDAQ,
}

impl Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Market::KOSPI => write!(f, "KOSPI"),
            Market::KOSDAQ => write!(f, "KOSDAQ"),
        }
    }
}

impl TryFrom<&str> for Market {
    type Error = anyhow::Error;

    fn try_from(code: &str) -> Result<Self, Self::Error> {
        match code {
            "1" => Ok(Market::KOSPI),
            "2" => Ok(Market::KOSDAQ),
            _ => Err(anyhow::anyhow!("Invalid market code: {}", code)),
        }
    }
}

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
pub struct Order {
    id: i64,
    symbol: String,
    quantity: i64,
    price: i64,
    action: OrderAction,
    order_type: OrderType,
}

impl Order {
    fn new(
        id: i64,
        symbol: String,
        quantity: i64,
        price: i64,
        action: OrderAction,
        order_type: OrderType,
    ) -> Self {
        Self {
            id,
            symbol,
            quantity,
            price,
            action,
            order_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tick {
    pub price: String,
    #[serde(rename = "cvolume")]
    pub volume: String,
    #[serde(rename = "shcode")]
    pub ticker: String,
}

impl Tick {
    pub fn new(ticker: String, price: String, volume: String) -> Self {
        Self {
            price,
            volume,
            ticker,
        }
    }
}

impl Display for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ticker: {}, price: {}, volume: {}",
            self.ticker, self.price, self.volume
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    #[serde(rename = "expcode")]
    pub ticker: String,
    #[serde(rename = "janqty")]
    pub quantity: i64,
    #[serde(rename = "appamt")]
    evaluation_price: f64,
    #[serde(rename = "pamt")]
    pub average_price: f64,
    #[serde(rename = "dtsunik")]
    profit: f64,
    #[serde(rename = "sunikrt")]
    rate_of_return: String,
    #[serde(rename = "fee")]
    fee: f64,
    #[serde(rename = "tax")]
    tax: f64,
}

#[async_trait]
pub trait Broker: Send + Sync {
    async fn get_tickers(&self) -> Result<HashMap<String, Market>>;
    async fn get_tick_data(&self, ticker: &str) -> Result<()>;
    async fn get_balance(&self) -> Result<i64>;
    async fn get_positions(&self) -> Result<Vec<Position>>;
    async fn order_cancel(&self, order: Order) -> Result<()>;
    async fn get_access_token(&self) -> Result<String>;
    async fn connect_websocket(&self, token: tokio_util::sync::CancellationToken) -> Result<Receiver<Tick>>;
    async fn order(
        &self,
        ticker: &str,
        amount: i64,
        price: i64,
        order_action: OrderAction,
        order_type: OrderType,
    ) -> Result<Order>;
}
