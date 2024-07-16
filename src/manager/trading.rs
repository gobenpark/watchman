use crate::broker;
use crate::strategies::strategy_base::Strategy;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use crate::strategies::strategy_base::{OrderDecision,OrderType};
use std::sync::Arc;
use tokio::join;
use tokio::sync::Mutex;

#[async_trait]
pub trait OrderExecutor: Send + Sync {
    async fn execute_buy(&self, symbol: &str, quantity: i32) -> Result<()>;
    async fn execute_sell(&self, symbol: &str, quantity: i32) -> Result<()>;
}

pub struct TradingManager {
    strategies: Vec<Arc<Mutex<Box<dyn Strategy>>>>,
    client: Arc<dyn broker::Broker>,
}

impl TradingManager {
    pub fn new(client: broker::lssec::LsSecClient) -> Self {
        Self {
            strategies: Vec::new(),
            client: Arc::new(client),
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(Arc::new(Mutex::new(strategy)));
    }

    pub async fn get_all_targets(&self) -> Vec<String> {
        // get all targets from strategies
        let mut targets = vec![];
        for strategy in &self.strategies {
            let target = {
                let strategy = strategy.lock().await;
                strategy.get_targets()
            };
            targets.extend(target);
        }
        targets
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tasks = Vec::new();

        for strategy in &self.strategies {
            let strategy = strategy.clone();
            let client = self.client.clone();

            let task = tokio::spawn(async move {
                let targets = {
                    let strategy = strategy.lock().await;
                    strategy.get_targets()
                };

                let mut inner_tasks = Vec::new();

                for target in targets {
                    log::info!("watch {}",target);
                    let mut real_data = client.get_tick_data(&target).await?;
                    let strategy_clone = strategy.clone();
                    let client = client.clone();
                    let inner_task = tokio::spawn(async move {
                        while let Some(tick) = real_data.recv().await {
                            let decision = {
                                let strategy = strategy_clone.lock().await;
                                strategy.evaluate_tick(&tick).await
                            };

                            Self::execute_decision(&decision.expect("not exist decision"),&client).await;
                        }
                    });

                    inner_tasks.push(inner_task);
                }
                for ta in inner_tasks{
                    ta.await?;
                }
                Ok::<(), anyhow::Error>(())
            });
            tasks.push(task);
        }

        for task in tasks{
            task.await??;
        }
        Ok(())
    }
    async fn execute_decision(
        decision: &OrderDecision,
        order_executor: &Arc<dyn broker::Broker>
    ) {
        println!("{}",decision);
        // if let Ok(()) = order_executor.execute_order(decision).await {
        //     if let Ok(mut pm) = position_manager.lock().await {
        //         let quantity = match decision.order_type {
        //             OrderType::Buy => decision.quantity as i32,
        //             OrderType::Sell => -(decision.quantity as i32),
        //         };
        //         pm.update_position(&decision.symbol, quantity);
        //     }
        // }
    }
}
