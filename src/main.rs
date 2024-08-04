
use std::env;
use std::sync::Arc;
use std::thread::sleep;
use teloxide::prelude::*;
use tokio::signal;
mod broker;
mod manager;
pub mod schema;
mod strategies;
mod api;
mod model;

use anyhow::Result;
use tokio::task::JoinHandle;
// use tokio_stream::StreamExt;
use crate::broker::{Broker};
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
    let client = api::lssec::LsSecClient::new(key, secret);
    let pcli = Arc::new(client.clone());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let broker = Arc::new(Broker::new(Box::new(client), database_url));
    let mut manager = TradingManager::new(broker);
    let envelope = Envelope::new();
    let sample = strategies::sample::SampleStrategy::new();
    manager.add_strategy(Box::new(envelope));
    manager.add_strategy(Box::new(sample));

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
