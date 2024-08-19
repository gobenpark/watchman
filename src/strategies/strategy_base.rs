use crate::model::position::Position;
use crate::model::tick::Tick;
use async_trait::async_trait;
use crate::model::prelude::*;
use anyhow::Result;

#[async_trait]
pub trait Strategy: Send + Sync {
    fn get_id(&self) -> String;
    fn get_targets(&self) -> Vec<String>;
    async fn evaluate_tick(
        &self,
        tick: &Tick,
        position: Option<Position>,
    ) -> Result<Option<NewOrder>>;
}

