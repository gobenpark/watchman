mod storage;

use std::env;
use std::thread::sleep;
use teloxide::prelude::*;
use tokio::signal;
mod broker;
mod manager;
mod model;
pub mod schema;
mod strategies;
use anyhow::Result;
use tokio::task::JoinHandle;
// use tokio_stream::StreamExt;
use crate::broker::{Broker, OrderAction, OrderType};
use dotenvy::dotenv;
use futures_util::{future, pin_mut, SinkExt, StreamExt, TryFutureExt, TryStreamExt};
use manager::trading::TradingManager;
use strategies::envelope::Envelope;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tonic::codegen::tokio_stream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    pyo3::prepare_freethreaded_python();
    tracing_subscriber::fmt::init();

    let key = env::var("LSSEC_KEY")?;
    let secret = env::var("LSSEC_SECRET")?;
    let client = broker::lssec::LsSecClient::new(key, secret);
    let mut manager = TradingManager::new(client);
    let envelope = Envelope::new();
    manager.add_strategy(Box::new(envelope));

    tokio::select! {
        result = manager.run() => {
            if let Err(e) = result {
                eprintln!("서버 에러: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            info!("shutdown server ...");
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        assert!(true);
        println!("test")
    }
}
