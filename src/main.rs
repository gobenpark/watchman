mod storage;
use teloxide::prelude::*;
use tokio::signal;
mod broker;
mod model;
pub mod schema;
mod manager;
mod strategies;
use anyhow::Result;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tonic::codegen::tokio_stream;
use tonic::{transport::Server, Request, Response, Status};

use manager::trading::TradingManager;
use strategies::envelope::Envelope;

static KEY: &str = "PS45hIFw1Xu7apziLQdUc4jNLazIPacQdqcX";
static SECRET: &str = "nzWMVzES7uvxUKyK68nmXb2cHHhOOg8o";
#[tokio::main]
async fn main() -> Result<()> {
    pyo3::prepare_freethreaded_python();
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let client = broker::lssec::LsSecClient::new(KEY.to_string(), SECRET.to_string());
    let mut manager = TradingManager::new(client);
    let envelope = Envelope::new();
    manager.add_strategy(Box::new(envelope));


    manager.run().await?;

    tokio::select! {
        result = manager.run() => {
            if let Err(e) = result {
                eprintln!("서버 에러: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            println!("종료 신호 받음, 서버 종료 중...");
        }
    }

    log::info!("Starting...");
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
