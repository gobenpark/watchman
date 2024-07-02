use crate::strategies::strategy_base::Strategy;
use anyhow::Result;
use broker::broker_service_client::BrokerServiceClient;
use tokio_stream::StreamExt;
use tonic::{transport::Server, Request, Response, Status};
use url::Url;

pub mod broker {
    tonic::include_proto!("broker");
}

pub struct TradingManager {
    broker: BrokerServiceClient<tonic::transport::Channel>,
    strategies: Vec<Box<dyn Strategy>>,
}

impl TradingManager {
    pub async fn new(broker_url: Url) -> Self {
        let mut client = BrokerServiceClient::connect(broker_url.as_str().to_string())
            .await
            .unwrap();
        Self {
            broker: client,
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }

    fn get_all_targets(&self) -> Vec<String> {
        self.strategies
            .iter()
            .map(|s| s.get_targets())
            .flatten()
            .collect()
    }

    pub async fn run(&mut self, target: Vec<String>) -> Result<()> {
        let response = self
            .broker
            .watch_order_transaction(Request::new(()))
            .await?;
        let mut stream = response.into_inner();

        for i in target {
            // self.broker.watch_transaction(Request::new(broker::WatchTransactionRequest{})).await.unwrap();
        }

        while let Some(message) = stream.next().await {
            println!("RESPONSE={:?}", message);
        }

        self.broker
            .watch_order_transaction(Request::new(()))
            .await
            .unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_create() -> Result<()> {
        let mut cli = TradingManager::new(Url::parse("http://[::1]:50051").unwrap());
        let result = cli.await.get_all_targets();
        println!("{:?}", result);

        Ok(())
    }
}
