use anyhow::Result;
use std::collections::HashMap;
use async_trait::async_trait;

// 예시 데이터 구조
pub struct MarketData {
    pub symbol: String,
    pub price: f64,
    pub volume: u64,
    // 기타 필요한 정보
}

pub struct HistoricalData {
    pub symbol: String,
    pub data: Vec<MarketData>,
    // 과거 데이터에 대한 추가 정보
}

// #[async_trait::async_trait]
// pub trait DataManager: Send + Sync {
//     /// 시장 데이터를 수집합니다.
//     async fn fetch_market_data(&self, symbols: &[&str]) -> Result<HashMap<String, MarketData>>;
//
//     /// 과거 데이터를 수집합니다.
//     async fn fetch_historical_data(&self, symbol: &str, start_date: &str, end_date: &str) -> Result<HistoricalData>;
//
//     /// 수집된 데이터를 저장합니다.
//     async fn store_data(&self, data: &MarketData) -> Result<()>;
//
//     /// 필요한 경우 데이터를 전처리합니다.
//     async fn preprocess_data(&self, data: &mut MarketData) -> Result<()>;
//
//     /// 특정 심볼에 대한 데이터를 조회합니다.
//     async fn get_data(&self, symbol: &str) -> Result<Option<MarketData>>;
//
//     /// 다수의 심볼에 대한 데이터를 조회합니다.
//     async fn get_bulk_data(&self, symbols: &[&str]) -> Result<HashMap<String, MarketData>>;
// }


pub struct DataManager;

impl DataManager {
    pub fn new() -> Self {
        Self
    }

    // async fn fetch_market_data(&self, symbols: &[&str]) -> Result<H>
}

#[cfg(test)]
mod test {
    use spider::tokio;
    use spider::website::Website;
    use std::default::Default;
    use tokio::time::Instant;
    use spider::configuration::WaitForIdleNetwork;
    use std::time::Duration;
    #[tokio::test]
    async fn test_data() {
        let mut website: Website = Website::new("https://rsseau.fr")
            .with_chrome_intercept(true, true)
            .with_wait_for_idle_network(Some(WaitForIdleNetwork::new(Some(Duration::from_secs(30)))))
            .with_caching(cfg!(feature = "cache"))
            .build()
            .unwrap();
        let mut rx2 = website.subscribe(16).unwrap();

        tokio::spawn(async move {
            while let Ok(page) = rx2.recv().await {
                println!("{:?}", page.get_url());
            }
        });

        let start = Instant::now();
        website.crawl().await;
        let duration = start.elapsed();

        let links = website.get_links();

        for link in links {
            println!("- {:?}", link.as_ref());
        }

        println!(
            "Time elapsed in website.crawl() is: {:?} for total pages: {:?}",
            duration,
            links.len()
        )






    }
}