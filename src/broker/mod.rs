use std::fmt::Display;
use std::sync::Arc;

use anyhow::Result;
use chrono::Duration;
use diesel::{Insertable, PgConnection};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, deadpool};
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::scoped_futures::ScopedFutureExt;
use log::info;
use moka::future::Cache;
use tokio::select;
use tokio::sync::mpsc::{channel, Receiver};
use tokio_util::sync::CancellationToken;
use tracing::error;
use crate::api::market::OrderResultType;

use crate::api::market::MarketAPI;
use crate::model::order::{Order, OrderAction, NewOrder, OrderType};
use crate::model::position::Position;
use crate::model::tick::Tick;
use crate::repository::Repository;
use crate::schema::orders::dsl::*;
use crate::schema::positions::dsl as positiondsl;
use crate::schema::positions::dsl::positions;


type DbPool = Pool<AsyncPgConnection>;

pub struct Broker{
    api: Box<dyn MarketAPI>,
    repository: Arc<Repository>,
    cache_position: Cache<String, Vec<Position>>,
}


impl Broker {
    pub fn new(api: Box<dyn MarketAPI>, repository: Arc<Repository>) -> Self {
        let cache = Cache::new(10_000);
        Self {api,repository,cache_position: cache}
    }

    pub async fn transaction(&self,ctx: CancellationToken,target: Vec<String>) -> Result<Receiver<Tick>> {
        let (tx,rx) = channel::<Tick>(100);
        let mut receiver = self.api.connect_websocket(ctx.clone()).await?;
        tokio::spawn(async move {
            loop {
                select! {
                    tk = receiver.recv() => {
                        if let Some(tk) = tk {
                            tx.send(tk).await.unwrap()
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
                                Ok(od) => {
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
        let mut po = self.repository.get_position(o.ticker().to_string(), o.strategy_id().to_string()).await?;
        let total = po.quantity + o.quantity as f64;
        let avg_price = (po.price * po.quantity + o.price * o.quantity as f64) / total;
        po.quantity = total;
        po.price = avg_price;
        self.repository.update_position(po).await
    }

    pub async fn execute_order(&self, o: Order) -> Result<Order> {
        let to = o.clone();
        match self.api.order(o).await {
            Ok(order) => {
                info!("Order executed: {:?}", order);
                let no = NewOrder::from(to);
                self.repository.add_order(no).await?;
                Ok(order)
            }
            Err(e) => {
                error!("Failed to execute order: {}", e);
                Err(diesel::result::Error::RollbackTransaction.into())
            }
        }
    }
}