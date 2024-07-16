use crate::strategies::strategy_base::{OrderDecision,OrderType};
use crate::strategies::strategy_base::Strategy;
use anyhow::Result;
use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::{Py, PyAny, PyResult, Python};
use crate::broker::Tick;

#[derive(Clone)]
pub struct Envelope {}

impl Envelope {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Strategy for Envelope {
    async fn on_market_data(&mut self) -> Result<()> {
        // println!("ticker: {}, price: {}",tick.ticker, tick.price);
        Ok(())
    }

    fn get_targets(&self) -> Vec<String> {
        pyo3::prepare_freethreaded_python();
        let py_app = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/strategies/envelope.py"
        ));
        Python::with_gil(|py| -> PyResult<Vec<String>> {
            let app: Py<PyAny> = PyModule::from_code_bound(py, py_app, "", "")?
                .getattr("Envolope")?
                .into();
            let instance = app.call0(py)?;
            let target: Vec<String> = instance.call_method0(py, "target")?.extract(py)?;
            Ok(target)
        })
        .expect("Failed to get targets from python")
    }

    async fn evaluate_tick(&self, tick: &Tick) -> Result<OrderDecision> {
        let symbol = &tick.ticker;
        let price: f64 = tick.price.parse()?;
        pyo3::prepare_freethreaded_python();
        let py_app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/strategies/envelope.py"
        ));
        let buy = Python::with_gil(|py| -> PyResult<bool> {
            let app: Py<PyAny> = PyModule::from_code_bound(py, py_app, "", "")?
                .getattr("Envolope")?
                .into();
            let instance = app.call0(py)?;
            // let target: bool = instance.call_method0(py, "buy")?.extract(py)?;
            let target: bool = instance.call_method1(py, "buy", (symbol,price))?.extract(py)?;
            Ok(target)
        })?;


        if buy {
            Ok(OrderDecision {
                order_type: OrderType::Buy,
                symbol: tick.ticker.clone(),
                quantity: 1,
                price: price,
                reason: "Buy signal detected".to_string(),
            })
        } else {
            Ok(OrderDecision {
                order_type: OrderType::Sell,
                symbol: tick.ticker.clone(),
                quantity: 1,
                price: price,
                reason: "Sell signal detected".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_envelope() {
        let env = Envelope::new();
    }

    #[tokio::test]
    async fn test_buy() -> Result<()>{
        let env = Envelope::new();
        // println!("{}",env.get_targets().len());
        let _ = env.evaluate_tick(&Tick::new("005930".to_string(), "100".to_string(), "100".to_string())).await?;
        Ok(())
    }
}
