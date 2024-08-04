use std::fmt;
use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Market {
    KOSPI,
    KOSDAQ,
}

impl Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Market::KOSPI => write!(f, "KOSPI"),
            Market::KOSDAQ => write!(f, "KOSDAQ"),
        }
    }
}

impl TryFrom<&str> for Market {
    type Error = anyhow::Error;

    fn try_from(code: &str) -> anyhow::Result<Self, Self::Error> {
        match code {
            "1" => Ok(Market::KOSPI),
            "2" => Ok(Market::KOSDAQ),
            _ => Err(anyhow::anyhow!("Invalid market code: {}", code)),
        }
    }
}