mod storage;
use teloxide::prelude::*;

mod api;
mod model;
pub mod schema;
mod service;
mod strategies;
use anyhow::Result;
use broker::broker_service_client::BrokerServiceClient;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;
use tonic::codegen::tokio_stream;
use tonic::{transport::Server, Request, Response, Status};
pub mod broker {
    tonic::include_proto!("broker");
}

use service::strategy_manager::StrategyManager;
static KEY: &str = "PS45hIFw1Xu7apziLQdUc4jNLazIPacQdqcX";
static SECRET: &str = "nzWMVzES7uvxUKyK68nmXb2cHHhOOg8o";
#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let client = api::lssec::LsSecClient::new(KEY.to_string(), SECRET.to_string());

    let mut sam = client.get_tick_data("005930").await?;

    let task1: JoinHandle<()> = tokio::spawn(async move {
        while let Some(result) = sam.recv().await {
            println!("005930: {}", result)
        }
    });

    let mut elec = client.get_tick_data("103590").await?;
    let task2: JoinHandle<()> = tokio::spawn(async move {
        while let Some(result) = elec.recv().await {
            println!("103590: {}", result)
        }
    });

    tokio::try_join!(task1, task2)?;

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
