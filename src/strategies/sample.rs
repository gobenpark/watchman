use crate::strategies::strategy_base::{OrderDecision, Strategy};
use crate::model::position::Position;
use crate::model::tick::Tick;
use async_trait::async_trait;

pub struct SampleStrategy {}

impl SampleStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Strategy for SampleStrategy {
    fn get_id(&self) -> String {
        "sample".to_string()
    }

    fn get_targets(&self) -> Vec<String> {
        todo!()
    }

    async fn evaluate_tick(
        &self,
        tick: &Tick,
        position: Option<Position>,
    ) -> anyhow::Result<OrderDecision> {
        todo!()
    }
}
