use crate::strategies::strategy_base::Strategy;

pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy>>
}

impl StrategyManager {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new()
        }
    }

    pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
        self.strategies.push(strategy);
    }

    pub async fn run(&mut self) {
        for strategy in self.strategies.iter_mut() {
            // strategy.
        }
    }
}