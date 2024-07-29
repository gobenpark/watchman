use std::collections::HashSet;
use crate::broker;
use crate::strategies::strategy_base::{OrderDecision, OrderType, Strategy};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

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
    pub fn new(client: impl broker::Broker + 'static) -> Self {
        Self {
            strategies: Vec::new(),
            client: Arc::new(client),
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
        let strategy_tasks = self.strategies.iter().map(|strategy| {
            let strategy = Arc::clone(strategy);
            let client = Arc::clone(&self.client);
            self.run_strategy(strategy, client)
        });

        join_all(strategy_tasks)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    async fn run_strategy(
        &self,
        strategy: Arc<Mutex<Box<dyn Strategy>>>,
        client: Arc<dyn broker::Broker>,
    ) -> Result<()> {
        let mut targets = {
            let strategy = strategy.lock().await;
            strategy.get_targets()
        };

        let positons = client.get_positions().await?;
        let ttargets: Vec<String> = positons.iter().map(|p| {
            p.ticker.clone()
        }).collect();
        targets.extend(ttargets);
        let new_symbols: HashSet<String> = targets.into_iter().collect();
        targets = new_symbols.into_iter().collect();

        let target_tasks = targets
            .into_iter()
            .map(|target| self.watch_target(Arc::clone(&strategy), Arc::clone(&client), target));

        join_all(target_tasks)
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    async fn watch_target(
        &self,
        strategy: Arc<Mutex<Box<dyn Strategy>>>,
        client: Arc<dyn broker::Broker>,
        target: String,
    ) -> Result<()> {
        info!("Watching {}", target);
        let mut tick_data = client.get_tick_data(&target).await?;

        while let Some(tick) = tick_data.recv().await {
            let decision = {
                let strategy = strategy.lock().await;
                let p = client.get_position(tick.ticker.as_str()).await;
                strategy
                    .evaluate_tick(&tick, p)
                    .await
                    .context("Failed to evaluate tick")?
            };
            let client = client.clone();
            self.execute_decision(&decision, client).await;
        }

        Ok(())
    }

    async fn execute_decision(
        &self,
        decision: &OrderDecision,
        client: Arc<dyn broker::Broker>,
    ) -> Result<()> {
        match decision.order_type {
            OrderType::Buy => {
                client
                    .order(
                        &decision.symbol,
                        1,
                        decision.price as i64,
                        broker::OrderAction::Buy,
                        broker::OrderType::Market,
                        false,
                    )
                    .await
                    .context("Failed to execute buy order")?;
                log::info!("decision: {}",decision);
                // client.execute_buy(&decision.symbol, decision.quantity).await
                //     .context("Failed to execute buy order")?;
            }
            OrderType::Sell => {
                client
                    .order(
                        &decision.symbol,
                        1,
                        decision.price as i64,
                        broker::OrderAction::Sell,
                        broker::OrderType::Market,
                        true,
                    )
                    .await
                    .context("Failed to execute buy order")?;
                log::info!("decision: {}",decision);
            }
            OrderType::Hold => {}
        }

        Ok(())
    }
}
