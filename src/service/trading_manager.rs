use broker::broker_service_client::BrokerServiceClient;
use tonic::{transport::Server, Request, Response, Status};
use url::Url;
use anyhow::Result;
use tokio_stream::StreamExt;


pub mod broker {
    tonic::include_proto!("broker");
}

pub struct TradingManager {
    broker: BrokerServiceClient<tonic::transport::Channel>,
}


impl TradingManager {
    pub async fn new(broker_url: Url) -> Self {
        let mut client = BrokerServiceClient::connect(broker_url.as_str().to_string()).await.unwrap();
        Self {
            broker: client,
        }
    }

    pub async fn run(&mut self,target: Vec<String>) -> Result<()> {
        let response = self.broker.watch_order_transaction(Request::new(())).await?;
        let mut stream = response.into_inner();

        for i in target {

            self.broker.watch_transaction(Request::new(broker::WatchTransactionRequest{})).await.unwrap();
        }

        while let Some(message) = stream.next().await {
            println!("RESPONSE={:?}", message);
        }


        self.broker.watch_order_transaction(Request::new(())).await.unwrap();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_create() -> Result<()>{
        let mut cli = TradingManager::new(Url::parse("http://[::1]:50051").unwrap());

        cli.await.run();

        Ok(())
    }
}



