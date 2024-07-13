use pyo3::{Py, PyAny, PyResult, Python};
use pyo3::prelude::*;
use crate::strategies::strategy_base::Strategy;
use async_trait::async_trait;
use anyhow::Result;
use crate::model;

pub struct Envelope {
}

impl Envelope {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Strategy for Envelope {
    async fn on_market_data(&mut self,tick: model::tick::Tick) -> Result<()> {
        println!("ticker: {}, price: {}",tick.ticker, tick.price);
        Ok(())
    }

    fn get_targets(&self) -> Vec<String> {
        pyo3::prepare_freethreaded_python();
        let py_app = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/strategies/envelope.py"));
        Python::with_gil(|py| -> PyResult<Vec<String>> {
            let app: Py<PyAny> = PyModule::from_code_bound(py, py_app, "", "")?
                .getattr("Envolope")?
                .into();
            let instance = app.call0(py)?;
            let target: Vec<String> = instance.call_method0(py,"target")?.extract(py)?;
            Ok(target)
        }).expect("Failed to get targets from python")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_envelope(){
        let env = Envelope::new();
        let result = env.get_targets();
        for i in result {
           println!("{}", i);
        }
    }
}