pub struct Position {
    ticker: String,
    amount: i64,
    avg_price: f64,
}

impl Position {
    fn new(ticker: String, amount: i64, avg_price: f64) -> Self {
        Position {
            ticker,
            amount,
            avg_price,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_data() {
        let p = Position::new("AAPL".to_string(), 100, 100.0);
        println!("{}", p.ticker)
    }
}
