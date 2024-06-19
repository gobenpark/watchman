mod storage;
use teloxide::prelude::*;

pub mod schema;
mod service;
mod model;
mod strategies;
use tonic::{transport::Server, Request, Response, Status};
use broker::broker_service_client::BrokerServiceClient;
use anyhow::Result;
use tonic::codegen::tokio_stream;
use tokio_stream::StreamExt;
pub mod broker {
    tonic::include_proto!("broker");
}


use service::strategy_manager::StrategyManager;


#[tokio::main]
async fn main() -> Result<()>{
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut client = BrokerServiceClient::connect("http://[::1]:50051").await.unwrap();
    let response = client.watch_order_transaction(Request::new(())).await?;

    let mut stream = response.into_inner();

    while let Some(message) = stream.next().await {
        println!("RESPONSE={:?}", message);
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