use std::sync::Arc;
use crate::broker;
use crate::broker::Broker;

struct PositionManager  {
    client: Arc<dyn Broker>,
    positions: Vec<broker::Position>,
}

impl PositionManager {
    pub fn new(client: Arc<dyn Broker>) -> Self {
        Self {
            client,
            positions: Vec::new(),
        }
    }

    pub fn get_position(&self, symbol: &str) {

    }
}