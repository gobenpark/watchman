
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait Strategy: Send + Sync {
    async fn on_market_data(&mut self) -> Result<()>;

}