use anyhow::{anyhow, Context, Result};
use futures_util::stream::SplitSink;
use futures_util::{future, pin_mut, SinkExt, StreamExt, TryFutureExt, TryStreamExt};
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Once};
use std::thread::sleep;
use std::{fmt, time};
use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use teloxide::dptree::di::DependencySupplier;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::http;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
static INIT: Once = Once::new();

enum OrderAction {
    Buy,
    Sell,
}

enum OrderType {
    Limit,
    Market,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Market {
    KOSPI,
    KOSDAQ,
}

impl fmt::Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Market::KOSPI => write!(f, "KOSPI"),
            Market::KOSDAQ => write!(f, "KOSDAQ"),
        }
    }
}

impl TryFrom<&str> for Market {
    type Error = anyhow::Error;

    fn try_from(code: &str) -> Result<Self, Self::Error> {
        match code {
            "1" => Ok(Market::KOSPI),
            "2" => Ok(Market::KOSDAQ),
            _ => Err(anyhow::anyhow!("Invalid market code: {}", code)),
        }
    }
}

impl OrderAction {
    fn as_str(&self) -> &str {
        match self {
            OrderAction::Sell => "1",
            OrderAction::Buy => "2",
        }
    }
}

impl OrderType {
    fn as_str(&self) -> &str {
        match self {
            OrderType::Limit => "00",
            OrderType::Market => "03",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tick {
    price: String,
    #[serde(rename = "cvolume")]
    volume: String,
    #[serde(rename = "shcode")]
    ticker: String
}

impl Display for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ticker: {}, price: {}, volume: {}", self.ticker, self.price, self.volume)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Position {
    #[serde(rename = "expcode")]
    ticker: String,
    #[serde(rename = "janqty")]
    quantity: i64,
    #[serde(rename = "appamt")]
    evaluation_price: f64,
    #[serde(rename = "pamt")]
    average_price: f64,
    #[serde(rename = "dtsunik")]
    profit: f64,
    #[serde(rename = "sunikrt")]
    rate_of_return: String,
    #[serde(rename = "fee")]
    fee: f64,
    #[serde(rename = "tax")]
    tax: f64,
}

pub struct LsSecClient {
    key: String,
    secret: String,
    token: Arc<String>,
    api: Client,
    cache: Cache<String, String>,
    connect_socket: AtomicBool,
    tickers: Arc<OnceCell<HashMap<String, Market>>>,
    ws_sender: Arc<Mutex<Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>,
    tick_channels: Arc<Mutex<HashMap<String, Sender<Tick>>>>,
}




impl LsSecClient {
    pub fn new(key: String, secret: String) -> Self {
        let client = Client::new();
        Self {
            key,
            secret,
            token: Default::default(),
            api: client,
            cache: Cache::new(10_000),
            connect_socket: AtomicBool::new(false),
            tickers: Arc::new(OnceCell::new()),
            ws_sender: Arc::new(Mutex::new(None)),
            tick_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_tickers(&self) -> Result<HashMap<String, Market>> {
        let result = self.tickers.get_or_try_init(|| self.fetch_tickers()).await;
        result.cloned()
    }

    async fn fetch_tickers(&self) -> Result<HashMap<String, Market>> {
        let result = self
            .api_call(
                "/stock/etc",
                "t8436",
                &serde_json::json!({
                    "t8436InBlock": {
                        "gubun": "0"
                    }
                }),
            )
            .await?;

        let list = result
            .get("t8436OutBlock")
            .context("t8436OutBlock not found in response")?
            .as_array()
            .context("t8436OutBlock is not an array")?;

        let mut tickers = HashMap::new();
        for item in list {
            let shcode = item
                .get("shcode")
                .and_then(|v| v.as_str())
                .context("shcode not found or not a string")?;

            let gubun = item
                .get("gubun")
                .and_then(|v| v.as_str())
                .context("gubun not found or not a string")?;

            let market = Market::try_from(gubun).context("Failed to parse market")?;

            tickers.insert(shcode.to_string(), market);
        }

        Ok(tickers)
    }

    pub async fn connect_websocket(&self) -> Result<()> {
        let (ws_stream, _) = connect_async("wss://openapi.ls-sec.co.kr:9443/websocket").await?;
        let (write, mut read) = ws_stream.split();
        *self.ws_sender.lock().await = Some(write);

        let channels = self.tick_channels.clone();
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                if let Ok(message) = message {
                    if let Ok(json) = serde_json::from_str::<Value>(&message.to_string()) {
                        let data: Option<Tick> = json.get("body").and_then(|body| {
                            serde_json::from_value(body.clone()).ok()
                        });

                        match data {
                            Some(tick) => {
                                let mut channels = channels.lock().await;
                                if let Some(sender) = channels.get(&tick.ticker) {
                                    let _ = sender.send(tick).await;
                                }
                            }
                            None => {
                                println!("{:?}", json);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }

    pub async fn get_tick_data2(&self, ticker: &str) -> Result<(Receiver<Tick>)> {
        let mut channels = self.tick_channels.lock().await;

        if !channels.contains_key(ticker) {
            let (tx, rx) = channel::<Tick>(100); // 버퍼 크기는 필요에 따라 조정
            channels.insert(ticker.to_string(), tx);
            let mut sender = self.ws_sender.lock().await;
            if sender.is_none() {
                drop(sender);
                self.connect_websocket().await?;
            }

            let tickermap = self.get_tickers().await?;

            let tickers = tickermap
                .get(ticker)
                .context("invalid ticker")?;

            let data = serde_json::json!({
                "header": {
                    "token": self.get_access_token().await?,
                    "tr_type": "3"
                },
                "body": {
                    "tr_cd": || -> &'static str {
                match tickers {
                    Market::KOSPI => "S3_",
                    Market::KOSDAQ => "K3_",
                }
            }(),
                    "tr_key": ticker
                }
            });

            let mut sender = self.ws_sender.lock().await;
            if let Some(sender) = sender.as_mut() {
                sender.send(Message::Text(data.to_string())).await?;
            }
            Ok(rx)
        } else {
            return Err(anyhow::anyhow!("Already subscribed"));
        }
    }

    pub async fn get_tick_data(&self) {
        let (ws_stream, _) = connect_async("wss://openapi.ls-sec.co.kr:9443/websocket")
            .await
            .expect("Failed to connect");
        let (mut write, read) = ws_stream.split();

        if !self
            .connect_socket
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            tokio::spawn(async move {
                read.for_each(|message| async {
                    let data = message.unwrap().into_data();
                    tokio::io::stdout().write_all(&data).await.unwrap();
                })
                .await
            });
            self.connect_socket
                .store(true, std::sync::atomic::Ordering::Relaxed)
        }
        let data = &serde_json::json!({
            "header": {
                "token": self.get_access_token().await.unwrap(),
                "tr_type": "3"
            },
            "body": {
                "tr_cd": "K3_",
                "tr_key": "086520"
            }
        });

        sleep(time::Duration::from_secs(3));
        write.send(Message::Text(data.to_string())).await.unwrap();
    }

    async fn get_access_token(&self) -> Result<String> {
        if let Some(token) = self.cache.get("access_token").await {
            return Ok(token.trim_matches('"').to_string());
        }

        let result = self
            .api
            .post("	https://openapi.ls-sec.co.kr:8080/oauth2/token")
            .form(&[
                ("grant_type", "client_credentials"),
                ("appkey", &self.key),
                ("appsecretkey", &self.secret),
                ("scope", "oob"),
            ])
            .send()
            .await?
            .json::<Value>()
            .await?;

        let token = result
            .get("access_token")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("No access token"))?
            .trim_matches('"')
            .to_string();

        self.cache
            .insert("access_token".to_string(), token.clone())
            .await;

        Ok(token)
    }

    async fn api_call(
        &self,
        path: &str,
        tr_cd: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let token = self.get_access_token().await?;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", token).parse()?);
        headers.insert("tr_cd", tr_cd.parse()?);
        headers.insert("tr_cont", "N".parse()?);

        self.api
            .post(format!("https://openapi.ls-sec.co.kr:8080{}", path))
            .headers(headers)
            .json(body)
            .send()
            .await?
            .json()
            .await
            .context("Failed to parse API response")
    }

    async fn get_balance(&self) -> Result<i64> {
        let result = self
            .api_call(
                "/stock/accno",
                "CSPAQ12200",
                &serde_json::json!({
                    "CSPAQ12200InBlock": {
                        "RecCnt": 1,
                        "MgmtBrnNo": "1",
                        "BalCreTp": "1"
                    }
                }),
            )
            .await?;
        result
            .get("CSPAQ12200OutBlock2")
            .and_then(|block| block.get("MnyOrdAbleAmt"))
            .and_then(|amt| amt.as_i64())
            .context("Failed to get balance")
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        let result = self
            .api_call(
                "/stock/accno",
                "t0424",
                &serde_json::json!({
                    "t0424InBlock": {
                        "prcgb": "",
                        "chegb": "",
                        "dangb": "",
                        "charge": "",
                        "cts_expcode": ""
                    }
                }),
            )
            .await?;

        let positions = result
            .get("t0424OutBlock1")
            .context("t0424OutBlock1 not found in response")?
            .as_array()
            .context("t0424OutBlock1 is not an array")?
            .iter()
            .map(|x| serde_json::from_value(x.clone()))
            .collect::<Result<Vec<Position>, _>>()
            .context("Failed to deserialize positions")?;

        Ok(positions)
    }

    async fn order(
        &self,
        ticker: &str,
        amount: i64,
        price: i64,
        order_action: OrderAction,
        order_type: OrderType,
    ) -> Result<i64> {
        let body = serde_json::json!({
            "CSPAT00601InBlock1": {
                "IsuNo": format!("A{}", ticker),
                "OrdQty": amount,
                "OrdPrc": price,
                "BnsTpCode": order_action.as_str(),
                "OrdprcPtnCode": order_type.as_str(),
                "MgntrnCode": "000",
                "LoanDt": "",
                "OrdCndiTpCode": "0"
            }
        });

        let result = self.api_call("/stock/order", "CSPAT00601", &body).await?;

        result
            .get("CSPAT00601OutBlock1")
            .and_then(|block| block.get("OrdNo"))
            .and_then(|ord_no| ord_no.as_i64())
            .context("Failed to get order number")
    }

    async fn order_cancel(&self, order_number: i64, ticker: &str, amount: i64) -> Result<()> {
        let body = serde_json::json!({
            "CSPAT00800InBlock1": {  // 정확한 tr_cd를 사용해야 합니다. 여기서는 예시로 CSPAT00800을 사용했습니다.
                "OrgOrdNo": order_number,
                "IsuNo": format!("A{}", ticker),
                "OrdQty": amount,
            }
        });

        self.api_call("/stock/order", "CSPAT00800", &body).await?; // tr_cd를 올바르게 수정해야 합니다.

        Ok(())
    }

    async fn cache_test(&self) {
        self.cache
            .insert("key".to_string(), "123".to_string())
            .await;
        let result = self.cache.get(&"key".to_string()).await;
        println!("{:?}", result);
    }
}

#[cfg(test)]
mod test {
    static KEY: &str = "PS45hIFw1Xu7apziLQdUc4jNLazIPacQdqcX";
    static SECRET: &str = "nzWMVzES7uvxUKyK68nmXb2cHHhOOg8o";
    static TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJ0b2tlbiIsImF1ZCI6ImZlY2YzZGFlLWZkMjQtNGMwNy1iZjJlLTdmYjYxYjdjZDgzYiIsIm5iZiI6MTcxOTc5NzQ2NiwiZ3JhbnRfdHlwZSI6IkNsaWVudCIsImlzcyI6InVub2d3IiwiZXhwIjoxNzE5ODcxMTk5LCJpYXQiOjE3MTk3OTc0NjYsImp0aSI6IlBTNDVoSUZ3MVh1N2FwemlMUWRVYzRqTkxheklQYWNRZHFjWCJ9.K5j0SV4BLfV573jObRPy3pV03mQQ36FpL7twgYJvJC8Y3hUHImFO0NFk0_dHt1v6YlkPQWBUYP_H5OEFZm522Q";

    use super::*;
    use serde_json::Value::String;
    use std::collections::HashMap;
    use teloxide::types::CountryCode::LS;
    #[tokio::test]
    async fn test() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let token = client.get_access_token().await.unwrap();
        // let token = client.get_access_token().await.unwrap();
        println!("{:?}", token)
    }

    #[tokio::test]
    async fn test_cache() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        client.cache_test().await;
    }

    #[tokio::test]
    async fn test_get_positions() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let positions = client.get_positions().await;
        for i in positions.unwrap() {
            println!("{:?}", i)
        }
    }

    #[tokio::test]
    async fn test_order() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
    }

    #[tokio::test]
    async fn test_get_balance() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let balance = client.get_balance().await;
        println!("{:?}", balance)
    }

    #[tokio::test]
    async fn test_once() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        client.get_tick_data().await;
    }

    #[tokio::test]
    async fn test_get_tickers() {
        let mut client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let map = client.get_tickers().await;
        let map = client.get_tickers().await;
        let map = client.get_tickers().await;
    }

    #[tokio::test]
    async fn test_websocket_connect() {
        let mut client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        client.get_tick_data2("086520").await;
        // let _ = client.connect_websocket().await;
    }
}
