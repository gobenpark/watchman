use crate::broker;
use crate::broker::Tick;
use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Display;

#[async_trait]
pub trait Strategy: Send + Sync {
    fn get_targets(&self) -> Vec<String>;
    async fn evaluate_tick(
        &self,
        tick: &Tick,
        position: Option<broker::Position>,
    ) -> Result<OrderDecision>;
}

#[derive(Debug, Clone)]
pub struct OrderDecision {
    pub order_type: OrderType,
    pub symbol: String,
    pub quantity: u32,
    pub price: f64,
    pub reason: String,
}

impl Display for OrderDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "symbol: {}, order_type: {:?}, quantity: {}, price: {}, reason: {}",
            self.symbol, self.order_type, self.quantity, self.price, self.reason
        )
    }
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Buy,
    Sell,
    Hold,
}
