use crate::model::tick::Tick;
use crate::model::position::Position;
use crate::strategies::strategy_base::Strategy;
use crate::strategies::strategy_base::{OrderDecision, OrderType};
use anyhow::Result;
use async_trait::async_trait;
use pyo3::prelude::*;
use pyo3::{Py, PyAny, PyResult, Python};

pub struct Envelope {
    app: Py<PyAny>,
}

impl Envelope {
    pub fn new() -> Self {
        let py_app = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/strategies/envelope.py"
        ));
        let app = Python::with_gil(|py| -> Py<PyAny> {
            PyModule::from_code_bound(py, py_app, "", "")
                .unwrap()
                .getattr("Envolope")
                .unwrap()
                .into()
        });

        Self { app }
    }
}

#[async_trait]
impl Strategy for Envelope {
    fn get_id(&self) -> String {
        return "Envelope".to_string();
    }

    fn get_targets(&self) -> Vec<String> {
        Python::with_gil(|py| -> PyResult<Vec<String>> {
            let instance = self.app.call0(py)?;
            let target: Vec<String> = instance.call_method0(py, "target")?.extract(py)?;
            Ok(target)
        })
        .expect("Failed to get targets from python")
    }

    async fn evaluate_tick(
        &self,
        tick: &Tick,
        position: Option<Position>,
    ) -> Result<OrderDecision> {
        let symbol = &tick.ticker;
        let price: f64 = tick.price.parse()?;

        match position {
            Some(p) => {
                let sell = Python::with_gil(|py| -> PyResult<bool> {
                    let instance = self.app.call0(py)?;
                    let target: bool = instance
                        .call_method1(py, "sell", (symbol, price, p.price))?
                        .extract(py)?;
                    Ok(target)
                })?;

                if sell {
                    return Ok(OrderDecision {
                        order_type: OrderType::Sell,
                        symbol: tick.ticker.clone(),
                        quantity: 1,
                        price: price,
                        reason: "Buy signal detected".to_string(),
                    });
                }
                Ok(OrderDecision {
                    order_type: OrderType::Hold,
                    symbol: tick.ticker.clone(),
                    quantity: 1,
                    price: price,
                    reason: "Hold signal detected".to_string(),
                })
            }
            None => {
                let buy = Python::with_gil(|py| -> PyResult<bool> {
                    let instance = self.app.call0(py)?;
                    let target: bool = instance
                        .call_method1(py, "buy", (symbol, price))?
                        .extract(py)?;
                    Ok(target)
                })?;

                if buy {
                    return Ok(OrderDecision {
                        order_type: OrderType::Buy,
                        symbol: tick.ticker.clone(),
                        quantity: 1,
                        price: price,
                        reason: "Buy signal detected".to_string(),
                    });
                }
                Ok(OrderDecision {
                    order_type: OrderType::Hold,
                    symbol: tick.ticker.clone(),
                    quantity: 1,
                    price: price,
                    reason: "Hold signal detected".to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use polars::prelude::*;
    use polars_lazy::prelude::*;
    use polars_sql::*;
    use std::env;
    #[test]
    fn test_envelope() {
        let env = Envelope::new();
    }

    #[tokio::test]
    async fn test_buy() -> Result<()> {
        pyo3::prepare_freethreaded_python();
        let env = Envelope::new();
        println!("{}", env.get_targets().len());
        // let _ = env.evaluate_tick(&Tick::new("005930".to_string(), "100".to_string(), "100".to_string())).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_polar() {
        let df = df! {
            "column_a" => &[1, 2, 3, 4, 5],
            "column_b" => &[5, 4, 3, 2, 1]
        }
        .unwrap();
    }
}
