use crate::broker;
use crate::strategies::strategy_base::{OrderDecision, OrderType, Strategy};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::future::join_all;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
// use futures::{StreamExt};
use crate::model::tick::Tick;
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;
use tonic::codegen::Body;

#[async_trait]
pub trait OrderExecutor: Send + Sync {
    async fn execute_buy(&self, symbol: &str, quantity: i32) -> Result<()>;
    async fn execute_sell(&self, symbol: &str, quantity: i32) -> Result<()>;
}

pub struct TradingManager {
    strategies: Vec<Arc<Mutex<Box<dyn Strategy>>>>,
    broker: Arc<broker::Broker>,
}

impl TradingManager {
    pub fn new(broker: Arc<broker::Broker>) -> Self {
        Self {
            strategies: Vec::new(),
            broker,
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(Arc::new(Mutex::new(strategy)));
    }

    pub async fn get_all_targets(&self) -> Result<Vec<String>> {
        let mut targets = Vec::new();
        for strategy in &self.strategies {
            let strategy = strategy.lock().await;
            targets.extend(strategy.get_targets());
        }
        Ok(targets)
    }

    pub async fn run(&self) -> Result<()> {
        let (mut tx, mut rx) = tokio::sync::broadcast::channel::<Tick>(100);
        let cancel = CancellationToken::new();
        let socket_cancel = cancel.clone();

        let order_cancel = cancel.clone();
        self.broker.process_order(order_cancel).await?;
        let mut socket = self.broker.transaction(socket_cancel,&["005930", "005935", "103590"]).await?;
        let (decision_tx, mut decision_rx) = channel(100);
        let ttx = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = socket.recv().await {
                match ttx.send(msg) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to send tick data: {}", e);
                    }
                }
            }
        });

        for strategy in self.strategies.iter().cloned() {
            let mut tick_rx = rx.resubscribe();
            let decision_tx = decision_tx.clone();

            tokio::spawn(async move {
                while let Ok(tick) = tick_rx.recv().await {
                    let strategy = strategy.lock().await;
                    let id = strategy.get_id();

                    // let positions = position_manager
                    //     .get_positions()
                    //     .expect("Failed to get positions")
                    //     .into_iter()
                    //     .filter(|p| p.strategy_id == id)
                    //     .collect::<Vec<_>>();

                    let decision = OrderDecision {
                        order_type: OrderType::Hold,
                        symbol: tick.ticker.clone(),
                        quantity: 0,
                        price: 0.0,
                        reason: format!("strategy id: {}, tick: {:?}", id, tick),
                    };
                    println!("Decision: {:?}", decision);
                    let _ = decision_tx.send((tick, decision)).await;
                }
            });
        }

        tokio::select! {
            _ = async {
                while let Some((tick, decision)) = decision_rx.recv().await {
                    println!("Tick: {:?}, Decision: {:?}", tick, decision);
                }
            } => {}
        }

        Ok(())
    }
    //
    // async fn execute_decision(
    //     &self,
    //     decision: &OrderDecision,
    //     client: Arc<dyn broker::Broker>,
    // ) -> Result<()> {
    //     match decision.order_type {
    //         OrderType::Buy => {
    //             client
    //                 .order(
    //                     &decision.symbol,
    //                     1,
    //                     decision.price as i64,
    //                     broker::OrderAction::Buy,
    //                     broker::OrderType::Market,
    //                 )
    //                 .await
    //                 .context("Failed to execute buy order")?;
    //             log::info!("decision: {}", decision);
    //             // client.execute_buy(&decision.symbol, decision.quantity).await
    //             //     .context("Failed to execute buy order")?;
    //         }
    //         OrderType::Sell => {
    //             client
    //                 .order(
    //                     &decision.symbol,
    //                     1,
    //                     decision.price as i64,
    //                     broker::OrderAction::Sell,
    //                     broker::OrderType::Market,
    //                 )
    //                 .await
    //                 .context("Failed to execute buy order")?;
    //             log::info!("decision: {}", decision);
    //         }
    //         OrderType::Hold => {}
    //     }
    //
    //     Ok(())
    // }
}
