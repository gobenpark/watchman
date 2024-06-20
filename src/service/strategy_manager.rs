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

    pub fn get_all_targets(&self) -> Vec<String> {
        // get all targets from strategies
        let mut strategies = vec![];
        for strategy in &self.strategies {
            strategies.extend(strategy.get_targets());
        }
        strategies
    }

}