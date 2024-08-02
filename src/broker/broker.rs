use serde::{Deserialize, Serialize};
use crate::api::market::MarketAPI;
use crate::storage::postgres::PostgresStorage;
use tokio::sync::mpsc::Receiver;
use crate::broker::Tick;
use anyhow::Result;
use diesel::dsl::broadcast;
use tokio::select;
use tokio::sync::mpsc::channel;
use tokio_util::sync::CancellationToken;

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


pub struct Broker{
    api: Box<dyn MarketAPI>,
    storage: Box<PostgresStorage>,
}


impl Broker {
    pub fn new(api: Box<dyn MarketAPI>, storage: Box<PostgresStorage>) -> Self {
        Self {api, storage}
    }

    pub async fn transaction(&self,ctx: CancellationToken) -> Result<Receiver<Tick>> {
        let (tx,rx) = channel::<Tick>(100);
        let mut receiver = self.api.connect_websocket(ctx.clone()).await?;

        tokio::spawn(async move {
            loop {
                select! {
                    tk = receiver.recv() => {
                        if let Some(tk) = tk {
                            tx.send(tk).unwrap()
                        }
                    }
                    _ = ctx.cancelled() => {
                        break;
                    }
                }
            }
        });
        Ok(rx)
    }


}