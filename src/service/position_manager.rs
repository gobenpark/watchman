use std::collections::HashMap;

pub struct PositionManager {
    positions: HashMap<String, i32>,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }

    pub fn update_position(&mut self, symbol: &str, quantity: i32) {
        *self.positions.entry(symbol.to_string()).or_insert(0) += quantity;
    }

    pub fn get_position(&self, symbol: &str) -> i32 {
        *self.positions.get(symbol).unwrap_or(&0)
    }
}
