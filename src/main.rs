mod storage;
use teloxide::prelude::*;

mod api;
mod model;
pub mod schema;
mod service;
mod strategies;

use anyhow::Result;
use broker::broker_service_client::BrokerServiceClient;
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

    let client = api::lssec::LsSecClient::new(KEY.to_string(),SECRET.to_string());
    client.get_tick_data().await;
    // let manager = StrategyManager::new();
    //
    // let mut client = BrokerServiceClient::connect("http://[::1]:50051")
    //     .await
    //     .unwrap();
    // let response = client.watch_order_transaction(Request::new(())).await?;
    //
    // let mut stream = response.into_inner();
    //
    // while let Some(message) = stream.next().await {
    //     println!("RESPONSE={:?}", message);
    // }

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
