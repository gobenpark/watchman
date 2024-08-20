use std::sync::Arc;

use anyhow::Result;
use diesel_async::{AsyncPgConnection};
use diesel_async::pooled_connection::deadpool::{Pool};
use log::info;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio_util::sync::CancellationToken;
use tracing::error;
use crate::api::market::OrderResultType;

use crate::api::market::MarketAPI;
use crate::model::order::{Order, NewOrder};
use crate::model::position::Position;
use crate::model::tick::Tick;
use crate::repository::Repository;



pub struct Broker{
    api: Box<dyn MarketAPI>,
    repository: Arc<Repository>,
}


impl Broker {
    pub fn new(api: Box<dyn MarketAPI>, repository: Arc<Repository>) -> Self {
        Self {api,repository}
    }

    pub async fn transaction(&self,ctx: CancellationToken,target: Vec<String>) -> Result<Receiver<Tick>> {
        let (tx,rx) = channel::<Tick>(100);
        let mut receiver = self.api.connect_websocket(ctx.clone()).await?;
        tokio::spawn(async move {
            loop {
                select! {
                    tk = receiver.recv() => {
                        if let Some(tk) = tk {
                            tx.send(tk).await.expect("send error")
                        }
                    }
                    _ = ctx.cancelled() => {
                        break;
                    }
                }
            }
        });

        for i in target{
            self.api.subscribe(i).await?;
        }
        Ok(rx)
    }


    pub async fn process_order(&self, ctx: CancellationToken) -> Result<()> {
        let mut receiver = self.api.connect_websocket_order_transaction(ctx.clone()).await?;
        let repo = self.repository.clone();
        tokio::spawn(async move {
            loop {
                select! {
                Some(v) = receiver.recv() => {
                    match v.result {
                        OrderResultType::Success => {
                            let oid = v.id.parse::<i32>().unwrap();
                            let result = repo.accept_order(oid).await;
                            match result {
                                Ok(_) => {
                                    info!("Order accepted: {}", v.id);
                                }
                                Err(e) => {
                                    error!("Failed to update order: {}", e);
                                }
                            }
                        },
                            OrderResultType::Cancel => {
                                let oid = v.id.parse::<i32>().unwrap();
                                let result = repo.delete_order(oid).await;
                                match result {
                                    Ok(_) => {
                                        info!("Order cancelled: {}", v.id);
                                    }
                                    Err(e) => {
                                        error!("Failed to delete order: {}", e);
                                    }
                                }
                            }
                        _ => {}
                    }
                }
                _ = ctx.cancelled() => {
                    info!("Order processing cancelled");
                    break;
                }
            }
            }
        });

        Ok(())
    }


    async fn update_position(&self,o: Order) -> Result<Position>{
        let po = self.repository.get_position(o.ticker(), o.strategy_id().to_string()).await?;
        match po {
            Some(mut p) => {
                let total = p.quantity + o.quantity as f64;
                let avg_price = (p.price * p.quantity + o.price * o.quantity as f64) / total;
                p.quantity = total;
                p.price = avg_price;
                self.repository.update_position(p).await
            }
            None => {
                Err(anyhow::Error::msg("No position found"))
            }
        }

    }

    pub async fn execute_order(&self, o: NewOrder) -> Result<NewOrder> {
        match self.api.order(o).await {
            Ok(order) => {
                info!("Order executed: {:?}", order);
                self.repository.add_order(order.clone()).await?;
                Ok(order)
            }
            Err(e) => {
                error!("Failed to execute order: {}", e);
                Err(diesel::result::Error::RollbackTransaction.into())
            }
        }
    }
}