use std::collections::HashMap;
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use crate::model::order::{NewOrder, Order, OrderAction, OrderType};
use crate::model::market::Market;
use crate::model::position::Position;
use crate::model::tick::Tick;


#[derive(Debug)]
pub enum OrderResultType {
    //접수
    Wait,
    //성공
    Success,
    //취소
    Cancel,
    //정정
    Edit,
    //거부
    Denied,
}

#[derive(Debug)]
pub struct OrderResult {
    pub id: String,
    pub result: OrderResultType,
}

#[async_trait]
pub trait MarketAPI: Send + Sync {
    async fn get_tickers(&self) -> anyhow::Result<HashMap<String, Market>>;
    async fn subscribe(&self, ticker: String) -> anyhow::Result<()>;
    async fn get_balance(&self) -> anyhow::Result<i64>;
    async fn get_positions(&self) -> anyhow::Result<Vec<Position>>;
    async fn order_cancel(&self, order: Order) -> anyhow::Result<()>;
    async fn get_access_token(&self) -> anyhow::Result<String>;
    async fn connect_websocket(
        &self,
        ctx: CancellationToken,
    ) -> anyhow::Result<Receiver<Tick>>;
    async fn order(
        &self,
        order: NewOrder,
    ) -> anyhow::Result<NewOrder>;
    async fn connect_websocket_order_transaction(
        &self,
        token: CancellationToken,
    ) -> anyhow::Result<Receiver<OrderResult>>;
}
