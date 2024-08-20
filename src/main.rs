
use std::env;
use std::sync::Arc;
use tokio::signal;
mod broker;
mod manager;
pub mod schema;
mod strategies;
mod api;
mod model;
mod repository;

use anyhow::Result;
use crate::broker::{Broker};
use dotenvy::dotenv;
use manager::trading::TradingManager;
use strategies::envelope::Envelope;
use tracing::{info};
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
    let repository = Arc::new(repository::Repository::new(database_url)?);
    let broker = Arc::new(Broker::new(Box::new(client), repository.clone()));
    let mut manager = TradingManager::new(broker, repository.clone());
    let envelope = Envelope::new(repository.clone());
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
