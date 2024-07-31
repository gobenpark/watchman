use std::sync::Arc;
use crate::broker;
use crate::broker::Broker;
use crate::storage::postgres::PostgresStorage;
use crate::position::Position;
use anyhow::Result;

#[derive(Clone)]
pub struct PositionManager  {
    client: Arc<dyn Broker>,
    positions: Vec<broker::Position>,
    storage: Arc<PostgresStorage>
}

impl PositionManager {
    pub fn new(client: Arc<dyn Broker>,storage: Arc<PostgresStorage>) -> Self {
        Self {
            client,
            positions: Vec::new(),
            storage
        }
    }

    pub fn add_position(&self, position: Position) -> Result<()> {
        self.storage.add_position(position)?;
        Ok(())
    }

    pub fn get_positions(&self) -> Result<Vec<Position>> {
        self.storage.get_positions()
    }
}