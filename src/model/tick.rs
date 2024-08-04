use std::fmt;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tick {
    pub price: String,
    #[serde(rename = "cvolume")]
    pub volume: String,
    #[serde(rename = "shcode")]
    pub ticker: String,
}

impl Tick {
    pub fn new(ticker: String, price: String, volume: String) -> Self {
        Self {
            price,
            volume,
            ticker,
        }
    }
}

impl Display for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ticker: {}, price: {}, volume: {}",
            self.ticker, self.price, self.volume
        )
    }
}