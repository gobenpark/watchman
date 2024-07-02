use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tick {
    ticker: String,
    price: f64,
}
