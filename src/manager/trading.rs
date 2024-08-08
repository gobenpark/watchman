use crate::broker;
use crate::strategies::strategy_base::Strategy;
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::future::join_all;
use std::collections::HashSet;
use std::sync::Arc;
use diesel::serialize::ToSql;
use tokio::signal;
use tokio::sync::mpsc::channel;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use crate::model::tick::Tick;
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;
use tonic::codegen::Body;
use crate::model::order::Order;
use crate::model::prelude::NewOrder;
use crate::repository::Repository;

pub struct TradingManager {
    strategies: Vec<Arc<Mutex<Box<dyn Strategy>>>>,
    broker: Arc<broker::Broker>,
    repository: Arc<Repository>,
}

impl TradingManager {
    pub fn new(broker: Arc<broker::Broker>,repository: Arc<Repository>) -> Self {
        Self {
            strategies: Vec::new(),
            broker,
            repository
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(Arc::new(Mutex::new(strategy)));
    }

    pub async fn get_all_targets(&self) -> Result<Vec<String>> {
        let result = self.repository.get_daily_caps().await?.iter().map(|c| c.ticker.clone()).collect();
        Ok(result)
    }

    pub async fn run(&self) -> Result<()> {

        let (mut tx, mut rx) = tokio::sync::broadcast::channel::<Tick>(10000000);
        let cancel = CancellationToken::new();
        self.broker.process_order(cancel.clone()).await?;
        let targets = self.get_all_targets().await?;
        let socket = self.broker.transaction(cancel.clone(),targets).await?;

        let (order_tx, mut order_rx) = channel::<NewOrder>(10000000);

        self.handle_socket_messages(socket, tx.clone()).await;
        self.spawn_strategy_handlers(tx, order_tx.clone()).await;
        self.handle_orders(order_rx).await;

        Ok(())
    }
    async fn handle_socket_messages(&self, mut socket: tokio::sync::mpsc::Receiver<Tick>, tx: tokio::sync::broadcast::Sender<Tick>) {
        tokio::spawn(async move {
            while let Some(msg) = socket.recv().await {
                if let Err(e) = tx.send(msg) {
                    error!("Failed to send tick data: {}", e);
                }
            }
        });
    }

    async fn spawn_strategy_handlers(&self, tx: tokio::sync::broadcast::Sender<Tick>, order_tx: tokio::sync::mpsc::Sender<NewOrder>) {
        for strategy in self.strategies.iter().cloned() {
            let mut tick_rx = tx.subscribe();
            let order_tx = order_tx.clone();
            let broker = self.broker.clone();
            let repo = self.repository.clone();


            tokio::spawn(async move {
                while let Ok(tick) = tick_rx.recv().await {
                    let mut strategy = strategy.lock().await;
                    let id = strategy.get_id();
                    let po = {
                        if let Ok(position) = repo.get_position(tick.ticker.clone(),id).await {
                            position
                        }else{
                            None
                        }
                    };
                    match strategy.evaluate_tick(&tick,po).await {
                        Ok(Some(order)) => {
                            if order_tx.send(order).await.is_ok() {
                                info!("Order sent: ");
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            error!("Failed to evaluate tick: {}", e);
                        }
                    }
                }
            });
        }
    }

    async fn handle_orders(&self, mut order_rx: tokio::sync::mpsc::Receiver<NewOrder>) {
        while let Some(order) = order_rx.recv().await {
            match self.broker.execute_order(order.clone()).await {
                Ok(_) => info!("Order executed: {}", order.ticker),
                Err(e) => error!("Failed to execute order: {}", e),
            }
        }
    }

}
