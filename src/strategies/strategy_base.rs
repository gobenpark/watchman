use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Strategy: Send + Sync {
    async fn on_market_data(&mut self) -> Result<()>;
    fn get_targets(&self) -> Vec<String>;
}
