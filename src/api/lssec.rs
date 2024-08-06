use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Once};
use std::thread::sleep;
use std::{fmt, time};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use diesel::row::NamedRow;
use futures_util::stream::SplitSink;
use futures_util::{future, pin_mut, FutureExt, SinkExt, StreamExt, TryFutureExt, TryStreamExt};
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::http;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use crate::api::market::{MarketAPI, OrderResult,OrderResultType};
use crate::broker;
use crate::model::position::Position;
use crate::model::order::{Order,OrderAction,OrderType};
use crate::model::tick::Tick;
use crate::model::market::Market;

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

impl Clone for LsSecClient {
    fn clone(&self) -> Self {
        LsSecClient {
            key: self.key.clone(),
            secret: self.secret.clone(),
            token: Arc::clone(&self.token),
            api: self.api.clone(),
            cache: self.cache.clone(),
            connect_socket: AtomicBool::new(self.connect_socket.load(Ordering::Relaxed)),
            tickers: Arc::clone(&self.tickers),
            ws_sender: Arc::clone(&self.ws_sender),
            tick_channels: Arc::clone(&self.tick_channels),
        }
    }
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


    async fn api_call(&self, path: &str, tr_cd: &str, body: &Value) -> Result<Value> {
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
}

#[async_trait]
impl MarketAPI for LsSecClient {
    async fn get_tickers(&self) -> Result<HashMap<String, Market>> {
        let result = self.tickers.get_or_try_init(|| self.fetch_tickers()).await;
        result.cloned()
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

    async fn connect_websocket_order_transaction(
        &self,
        token: CancellationToken,
    ) -> Result<Receiver<OrderResult>> {
        let (ws_stream, _) = connect_async("wss://openapi.ls-sec.co.kr:9443/websocket").await?;
        let (mut write, mut read) = ws_stream.split();

        for i in &["SC0", "SC1", "SC2", "SC3", "SC4"] {
            write
                .send(Message::text(
                    serde_json::json!({
                        "header": {
                            "token": self.get_access_token().await?,
                            "tr_type": "1"
                        },
                        "body": {
                            "tr_cd": i,
                            "tr_key": ""
                        }
                    })
                        .to_string(),
                ))
                .await?;
        }
        let (tx, rx) = channel::<OrderResult>(100);
        tokio::spawn(async move {
            tokio::select! {
                _ = read.map(|msg| {
                    if let Ok(message) = msg {
                        if let Ok(json) = serde_json::from_str::<Value>(&message.to_string()) {
                        if let Some(trcd) = json.get("header").and_then(|header| header.get("tr_cd")) {
                         let order_no = json
                            .get("body")
                            .and_then(|body| body.get("ordno"))
                            .and_then(|ordno| ordno.as_str())
                            .and_then(|s| Some(s.to_string()));
                            match trcd.as_str() {
                                Some("SC0") => {
                                        if let Some(order_no) = order_no {
                                            return Some(OrderResult{ id: order_no, result: OrderResultType::Wait })
                                        }
                                },
                                Some("SC1") => {
                                        if let Some(order_no) = order_no {
                                            return Some(OrderResult{ id: order_no, result: OrderResultType::Success })
                                        }
                                },
                                Some("SC2") => {
                                        if let Some(order_no) = order_no {
                                            return Some(OrderResult{ id: order_no, result: OrderResultType::Edit })
                                        }
                                },
                                Some("SC3") => {
                                        if let Some(order_no) = order_no {
                                            return Some(OrderResult{ id: order_no, result: OrderResultType::Cancel })
                                        }
                                },
                                Some("SC4") => {
                                    if let Some(order_no) = order_no {
                                            return Some(OrderResult{ id: order_no, result: OrderResultType::Denied })
                                        }
                                }
                                _ => {
                                        println!("none")
                                    }
                            }
                                }
                        }
                    }
                    None
                }).for_each(|msg| async {
                    match msg {
                        Some(message) => {
                            tx.send(message).await.unwrap();
                        },
                        None => {
                            info!("does not tick data")
                        }
                    }

                }) => {
                    info!("stop receive websocket tick data")
                }
                _ = token.cancelled() => {
                    info!("receive signal")
                }
            }
        });
        Ok(rx)
    }

    async fn connect_websocket(&self, token: CancellationToken) -> Result<Receiver<Tick>> {
        let (ws_stream, _) = connect_async("wss://openapi.ls-sec.co.kr:9443/websocket").await?;
        let (write, mut read) = ws_stream.split();
        *self.ws_sender.lock().await = Some(write);

        let (tx, rx) = channel::<Tick>(100);
        tokio::spawn(async move {
            tokio::select! {
                _ = read.map(|msg| {
                    if let Ok(message) = msg {
                        if let Ok(json) = serde_json::from_str::<Value>(&message.to_string()) {
                            if let Some(cd) = json.get("body").and_then(|body| {
                                serde_json::from_value::<Tick>(body.clone()).ok()
                            }) {
                                return Some(cd)
                            }
                        }
                    }
                    None
                }).for_each(|msg| async {
                    match msg {
                        Some(message) => {
                            tx.send(message).await.unwrap();
                        },
                        None => {
                            info!("does not tick data")
                        }
                    }

                }) => {
                    info!("stop receive websocket tick data")
                }
                _ = token.cancelled() => {
                    info!("receive signal")
                }
            }
        });
        Ok(rx)
    }

    async fn subscribe(&self, ticker: &str) -> Result<()> {
        let mut channels = self.tick_channels.lock().await;
        if !channels.contains_key(ticker) {
            let mut sender = self.ws_sender.lock().await;
            if sender.is_none() {
                return Err(anyhow::Error::msg("No websocket connection"));
            }
            drop(sender);
            let tickermap = self.get_tickers().await?;
            let tickers = tickermap.get(ticker).context("invalid ticker")?;
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
            Ok(())
        } else {
            return Err(anyhow::anyhow!("Already subscribed"));
        }
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

    async fn order_cancel(&self, order: Order) -> Result<()> {
        let body = serde_json::json!({
            "CSPAT00801InBlock1": {  // 정확한 tr_cd를 사용해야 합니다. 여기서는 예시로 CSPAT00800을 사용했습니다.
                "OrgOrdNo": order.id,
                "IsuNo": format!("A{}", order.ticker()),
                "OrdQty": order.quantity,
            }
        });

        self.api_call("/stock/order", "CSPAT00801", &body).await?; // tr_cd를 올바르게 수정해야 합니다.

        Ok(())
    }

    async fn order(
        &self,
        mut order: Order,
    ) -> Result<Order> {
        let body = serde_json::json!({
            "CSPAT00601InBlock1": {
                "IsuNo": format!("A{}", order.ticker()),
                "OrdQty": order.quantity,
                "OrdPrc": || -> i64 {
                    match order.order_type {
                        OrderType::Market => 0,
                        OrderType::Limit => order.price as i64,
                    }
                }(),
                "BnsTpCode": order.action.as_str(),
                "OrdprcPtnCode": order.order_type.as_str(),
                "MgntrnCode": "000",
                "LoanDt": "",
                "OrdCndiTpCode": "0"
            }
        });

        let result = self.api_call("/stock/order", "CSPAT00601", &body).await?;
        let id = result
            .get("CSPAT00601OutBlock2")
            .and_then(|block| block.get("OrdNo"))
            .and_then(|ord_no| ord_no.as_i64())
            .context("Failed to get order number")?;
        order.set_id(id as i32);
        Ok(order)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures_util::future::join_all;

    static KEY: &str = "PSA0cTqjeDE2hoNUclL2tiHgOLLQwzdkX43e";
    static SECRET: &str = "H0jIqEYl6cqNGr7CyyfHfN2Ns1hPiRR7";

    #[tokio::test]
    async fn test() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let token = client.get_access_token().await.unwrap();
        // let token = client.get_access_token().await.unwrap();
        println!("{:?}", token);
    }

    #[tokio::test]
    async fn test_get_positions() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let positions = client.get_positions().await.expect("");
        let result = positions.iter().find(|p| p.ticker == "030520").take();
        match result {
            Some(p) => {
                println!("{}", p.ticker)
            }
            None => {
                println!("None!")
            }
        }
    }

    #[tokio::test]
    async fn test_get_balance() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let balance = client.get_balance().await;
        println!("{:?}", balance)
    }

    #[tokio::test]
    async fn test_get_tickers() {
        let mut client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let map = client.get_tickers().await;
        let map = client.get_tickers().await;
        let map = client.get_tickers().await;
    }
    //
    // #[tokio::test]
    // async fn test_limit_orders() {
    //     let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
    //     tokio::time::sleep(std::time::Duration::from_secs(2));
    //
    //     let result = client
    //         .order("092190", 1, 4200, OrderAction::Buy, OrderType::Limit)
    //         .await
    //         .expect("error order");
    //     tokio::time::sleep(time::Duration::from_secs(2));
    //     client.order_cancel(result).await.expect("error cancel");
    //     tokio::time::sleep(time::Duration::from_secs(2));
    // }
    //
    // #[tokio::test]
    // async fn test_market_order() {
    //     let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
    //     tokio::time::sleep(time::Duration::from_secs(2)).await;
    //
    //     let result = client
    //         .order("092190", 1, 0, OrderAction::Buy, OrderType::Market)
    //         .await
    //         .expect("error order");
    //     tokio::time::sleep(time::Duration::from_secs(2)).await;
    //     client.order_cancel(result).await.expect("error cancel");
    //     tokio::time::sleep(time::Duration::from_secs(2)).await;
    // }

    #[tokio::test]
    async fn test_tick() {
        let client = Arc::new(LsSecClient::new(KEY.to_string(), SECRET.to_string()));

        let result = client.get_tickers().await.unwrap();
        let tk = CancellationToken::new();
        let mut sockets = client.connect_websocket(tk).await.unwrap();

        let subsclient = client.clone();
        tokio::spawn(async move { subsclient.subscribe("005930").await.unwrap() });

        while let Some(tick) = sockets.recv().await {
            println!("{:?}", tick)
        }
    }

    #[tokio::test]
    async fn test_sample() {
        let client = Arc::new(LsSecClient::new(KEY.to_string(), SECRET.to_string()));

        let tk = CancellationToken::new();
        if let Ok(mut rx) = client.connect_websocket_order_transaction(tk).await {
            while let Some(i)  = rx.recv().await {
                println!("{:?}",i)
            }
        }
    }
}
