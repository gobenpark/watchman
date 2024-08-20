use crate::model::position::Position;
use crate::model::prelude::*;
use crate::model::tick::Tick;
use crate::repository::Repository;
use crate::strategies::strategy_base::Strategy;
use anyhow::{Context, Result};
use async_trait::async_trait;
use moka::future::Cache;
use polars::prelude::*;
use pyo3::prelude::*;
use std::sync::Arc;
use tokio::sync::OnceCell;


const CACHE_SIZE: u64 = 10_000;

pub struct Envelope {
    targets: OnceCell<Vec<String>>,
    repository: Arc<Repository>,
    cache: Cache<String,DataFrame>
}

impl Envelope {
    pub fn new(repository: Arc<Repository>) -> Self {
        Self { targets: OnceCell::new(),repository, cache: Cache::new(CACHE_SIZE)}
    }
}

impl Envelope {
    async fn get_df(&self,tk: &str) -> Result<DataFrame> {
        let data = self.cache.try_get_with::<_,anyhow::Error>(tk.to_string(), async move {
            let dc = self.repository.get_ohlcv(tk).await?;
            let df = DataFrame::new(vec![
                Series::new("ticker", dc.iter().map(|x| x.ticker.clone()).collect::<Vec<String>>()),
                Series::new("open", dc.iter().map(|x| x.open).collect::<Vec<f64>>()),
                Series::new("close", dc.iter().map(|x| x.close).collect::<Vec<f64>>()),
                Series::new("high", dc.iter().map(|x| x.high).collect::<Vec<f64>>()),
                Series::new("low", dc.iter().map(|x| x.low).collect::<Vec<f64>>()),
                Series::new("volume", dc.iter().map(|x| x.volume).collect::<Vec<i32>>()),
                Series::new("datetime", dc.iter().map(|x| x.datetime).collect::<Vec<chrono::NaiveDateTime>>()),
            ])?;

            let frame = df.clone().lazy().filter(col("ticker").eq(lit(tk))).select([
                col("close").shift(Expr::from(1)).alias("prev_close"),
                ((col("close") - col("close").shift(Expr::from(1))) / col("close").shift(Expr::from(1))).alias("diff"),
                col("ticker"),
                col("close"),
                col("high"),
                col("close").rolling_mean(RollingOptionsFixedWindow {
                    window_size: 7,
                    min_periods: 7,
                    weights: None,
                    center: false,
                    fn_params: None,
                }).alias("ma7"),
                col("close").rolling_mean(RollingOptionsFixedWindow {
                    window_size: 20,
                    min_periods: 20,
                    weights: None,
                    center: false,
                    fn_params: None,
                }).alias("ma20"),
                col("close").rolling_mean(RollingOptionsFixedWindow {
                    window_size: 10,
                    min_periods: 10,
                    weights: None,
                    center: false,
                    fn_params: None,
                }).alias("ma10"),
                col("datetime")
            ]).with_column((col("ma20") * lit(1.4)).alias("upper")).with_column((col("high").gt(col("upper"))).alias("mask_upper_cross"))
              .collect().map_err(|e| anyhow::anyhow!("fail to collect lazy dataframe: {}",e))?;
            Ok(frame)
        }).await.map_err(|e| anyhow::anyhow!("Cache error: {}",e))?;
        Ok(data)
    }

    async fn buy(&self,tk: &str,current_price: f64) -> Result<bool>{
        let df = self.get_df(tk).await?;
        let moving7 = df["ma7"].f64()?;
        let last_moving7 = moving7.get(moving7.len() - 2).context("not found")?;
        let v = ((current_price - last_moving7)/last_moving7).abs() * 100.0;
        Ok(v < 0.5)
    }

    async fn sell(&self,tk: &str,order_price: f64, current_price: f64) -> Result<bool> {
        let df = self.get_df(tk).await?;
        if order_price > current_price {
            let moving20 = df.column("ma20")?.f64()?;
            let last_moving20 = moving20.last().context("cannot get last moving20 value")?;
            let v = ((current_price - last_moving20) / last_moving20).abs() * 100.0;
            Ok(v < 0.5)
        } else {
            let moving10 = df.column("ma10")?.f64()?;
            let last_moving10 = moving10.last().context("cannot get last moving10 value")?;
            let v = ((current_price - last_moving10) / last_moving10).abs() * 100.0;
            Ok(v < 0.5)
        }
    }
}

#[async_trait]
impl Strategy for Envelope {
    fn get_id(&self) -> String {
        return "Envelope".to_string();
    }


    fn get_targets(&self) -> Vec<String> { todo!()
    }

    async fn evaluate_tick(
        &self,
        tick: &Tick,
        position: Option<Position>,
    ) -> Result<Option<NewOrder>> {

        let price: f64 = tick.price.parse()?;

        match position {
            Some(p) => {
                let result = self.sell(tick.ticker.as_str(), p.price, tick.price.parse()?).await?;
                if result {
                    return Ok(Some(NewOrder{
                        id: 0,
                        ticker: tick.ticker.clone(),
                        quantity: 1,
                        price,
                        strategy_id: self.get_id(),
                        action: OrderAction::Sell,
                        order_type: OrderType::Market,
                    }))
                }
            }
            None => {
                let result = self.buy(tick.ticker.as_str(), tick.price.parse()?).await?;
                if result {
                    return Ok(Some(NewOrder{
                        id: 0,
                        ticker: tick.ticker.clone(),
                        quantity: 1,
                        price,
                        strategy_id: self.get_id(),
                        action: OrderAction::Buy,
                        order_type: OrderType::Limit,
                    }))
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use polars::prelude::*;

    #[tokio::test]
    async fn test_buy() -> Result<()> {
        pyo3::prepare_freethreaded_python();
        let repo = Repository::new("postgres://postgres:1q2w3e4r@192.168.0.58:55432/analyzer".to_string())?;
        let env = Envelope::new(Arc::new(repo));
        let df = env.get_df("005930").await;
        println!("{:?}",df);
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
