use std::{fmt, time};
use std::collections::{HashMap, VecDeque};
use std::ffi::CString;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::sync::{Arc, Once};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use diesel::row::NamedRow;
use futures_util::{future, pin_mut, SinkExt, StreamExt, TryFutureExt, TryStreamExt};
use futures_util::stream::SplitSink;
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use teloxide::dptree::di::DependencySupplier;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::sync::OnceCell;
use tokio_tungstenite::{
    connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream,
};
use tokio_tungstenite::tungstenite::handshake::client::generate_key;
use tokio_tungstenite::tungstenite::http;

use crate::broker;
use crate::broker::{Broker, Market, Order, OrderAction, OrderType, Position, Tick};

static INIT: Once = Once::new();

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
    position: Arc<OnceCell<Cache<String,Position>>>,
    orders: Arc<Mutex<VecDeque<Order>>>,
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
            position: self.position.clone(),
            orders: Arc::clone(&self.orders)
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
            position: Arc::new(OnceCell::new()),
            orders: Arc::new(Mutex::new(VecDeque::new()))
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

    pub async fn connection_trasaction(&self) -> Result<()> {
        let (ws_stream, _) = connect_async("wss://openapi.ls-sec.co.kr:29443/websocket").await?;
        let (mut write, mut read) = ws_stream.split();
        let token = self.get_access_token().await?.clone();
        write.send(Message::text(serde_json::json!({
            "header": {
                "token": token,
                "tr_type": "1"
            },
            "body": {
                "tr_cd": "SC0",
                "tr_key": ""
            }
        }).to_string())).await?;
        write.send(Message::text(serde_json::json!({
            "header": {
                "token": token,
                "tr_type": "1"
            },
            "body": {
                "tr_cd": "SC1",
                "tr_key": ""
            }
        }).to_string())).await?;
        write.send(Message::text(serde_json::json!({
            "header": {
                "token": token,
                "tr_type": "1"
            },
            "body": {
                "tr_cd": "SC2",
                "tr_key": ""
            }
        }).to_string())).await?;

        let orders = self.orders.clone();
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                if let Ok(message) = message {
                    if let Ok(json) = serde_json::from_str::<Value>(&message.to_string()) {

                        let trcd = json.get("header").and_then(|header| {
                            header.get("tr_cd")
                        });

                        if let Some(trcd) = trcd {
                            match trcd.as_str() {
                                Some(cd) => {
                                    match cd{
                                        "SC0" => {
                                            println!("접수됨")
                                        },
                                        "SC1" => {
                                            let orderNo = json.get("body")
                                                .and_then(|body| body.get("ordno"))
                                                .and_then(|ordno| ordno.as_str())
                                                .and_then(|s| s.parse::<i64>().ok());
                                            {
                                                let mut orders = orders.lock().await;
                                                // find order index
                                                let order = orders.iter().position(|o| o.id == orderNo.unwrap());
                                                match order {
                                                    Some(o) => {
                                                        orders.remove(o);
                                                    },
                                                    None => {}
                                                }
                                            }



                                        },
                                        "SC3" => {
                                        },
                                        _ => {}
                                    }
                                },
                                None => {}
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }


    async fn connect_websocket(&self) -> Result<()> {
        self.connection_trasaction().await?;
        let (ws_stream, _) = connect_async("wss://openapi.ls-sec.co.kr:9443/websocket").await?;
        let (write, mut read) = ws_stream.split();
        *self.ws_sender.lock().await = Some(write);

        let tickermap = self.get_tickers().await?.clone();
        let token = self.get_access_token().await?.clone();
        let ws_sender = self.ws_sender.clone();
        let channels = self.tick_channels.clone();

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                if let Ok(message) = message {
                    if let Ok(json) = serde_json::from_str::<Value>(&message.to_string()) {
                        let data: Option<Tick> = json
                            .get("body")
                            .and_then(|body| serde_json::from_value(body.clone()).ok());

                        match data {
                            Some(tick) => {
                                let mut channels = channels.lock().await;
                                let ticker = tick.ticker.clone();
                                if let Some(sender) = channels.get(&ticker) {
                                    // check error channel close then remove from channels
                                    if let Err(_) = sender.send(tick).await {
                                        let tickers = tickermap
                                            .get(&ticker)
                                            .expect("error get ticker from map");

                                        let data = serde_json::json!({
                                            "header": {
                                                "token": token,
                                                "tr_type": "4"
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
                                        let mut writer = ws_sender.lock().await;
                                        if let Some(writer) = writer.as_mut() {
                                            let _ = writer.send(Message::text(data.to_string()));
                                            channels.remove(&ticker);
                                        }
                                        drop(writer);
                                    }
                                }
                                drop(channels);
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


    async fn api_call(
        &self,
        path: &str,
        tr_cd: &str,
        body: &Value,
    ) -> Result<Value> {
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
impl Broker for LsSecClient {

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

    async fn get_tick_data(&self, ticker: &str) -> Result<(Receiver<Tick>)> {
        let mut channels = self.tick_channels.lock().await;
        if !channels.contains_key(ticker) {
            let (tx, rx) = channel::<Tick>(100); // 버퍼 크기는 필요에 따라 조정
            channels.insert(ticker.to_string(), tx);
            let mut sender = self.ws_sender.lock().await;
            if sender.is_none() {
                drop(sender);
                self.connect_websocket().await?;
            } else {
                drop(sender);
            }

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
            Ok(rx)
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

    async fn get_position(&self,symbol: &str) -> Option<Position> {

        let data: Result<&Cache<String, Position>> = self.position.get_or_try_init(|| async {
            log::info!("init position cache");
            let positions = self.get_positions().await?;
            let cache = Cache::new(10_000);
            for position in positions {
                cache.insert(position.ticker.to_string(),position).await
            }
            Ok(cache)
        }).await;

        match data {
            Ok(o) => {
                o.get(symbol).await
            },
            Err(e) => {
                None
            }
        }
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
                "IsuNo": format!("A{}", order.symbol),
                "OrdQty": order.quantity,
            }
        });

        self.api_call("/stock/order", "CSPAT00801", &body).await?; // tr_cd를 올바르게 수정해야 합니다.

        Ok(())
    }

    async fn order(
        &self,
        symbol: &str,
        amount: i64,
        price: i64,
        order_action: OrderAction,
        order_type: OrderType,
        force: bool
    ) -> Result<Order> {

        let od = {
            let orders = self.orders.lock().await;
            orders.iter().find(|o| o.symbol == symbol).map(|o| o.clone())
        };

        match od {
            Some(od) => {
                if !force {
                    return Ok(od.clone());
                }
            },
            None => {}
        }

        let body = serde_json::json!({
            "CSPAT00601InBlock1": {
                "IsuNo": format!("A{}", symbol),
                "OrdQty": amount,
                "OrdPrc": || -> i64 {
                    match order_type {
                        OrderType::Market => 0,
                        OrderType::Limit => price,
                    }
                }(),
                "BnsTpCode": order_action.as_str(),
                "OrdprcPtnCode": order_type.as_str(),
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

        let order = Order::new(id,symbol.to_string(),amount,price,order_action,order_type);
        let mut orders = self.orders.lock().await;
        let od = order.clone();
        orders.push_back(order);

        log::info!("current orders {:?}",orders);
        Ok(od)
    }
}

#[cfg(test)]
mod test {
    use futures_util::future::join_all;
    use super::*;

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
                println!("{}",p.ticker)
            },
            None => {
                println!("None!")
            }
        }
    }

    #[tokio::test]
    async fn test_order() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let result = client.get_position("005949").await;

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

    #[tokio::test]
    async fn test_websocket_connect() {
        let mut client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let data = client.get_position("030520").await;
        match data {
            Some(d) => {
                println!("{}",d.ticker)
            },
            None => {
                println!("none")
            }
        }
    }

    #[tokio::test]
    async fn test_limit_orders() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        client.connection_trasaction().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(2));

        let result = client.order("092190",1,4200,OrderAction::Buy,OrderType::Limit,false).await.expect("error order");
        tokio::time::sleep(time::Duration::from_secs(2));
        client.order_cancel(result).await.expect("error cancel");
        tokio::time::sleep(time::Duration::from_secs(2));
    }

    #[tokio::test]
    async fn test_market_order() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        client.connection_trasaction().await.unwrap();
        tokio::time::sleep(time::Duration::from_secs(2)).await;

        let result = client.order("092190",1,0,OrderAction::Buy,OrderType::Market,false).await.expect("error order");
        tokio::time::sleep(time::Duration::from_secs(2)).await;
        client.order_cancel(result).await.expect("error cancel");
        tokio::time::sleep(time::Duration::from_secs(2)).await;
    }

    #[tokio::test]
    async fn test_tick() {
        let client = Arc::new(LsSecClient::new(KEY.to_string(), SECRET.to_string()));

        let result = client.get_tickers().await.unwrap();
        let results = Arc::new(result);

        let mut sps = Vec::new();
        for (k,v) in results.iter(){
            let client = Arc::clone(&client);
            let result = Arc::clone(&results);
            let k = k.clone();
            let handler = tokio::task::spawn(async move {
                let mut data = client.get_tick_data(k.as_str()).await.unwrap();
                while let Some(tick) = data.recv().await {
                    println!("{:?}",tick);
                }
            });
            sps.push(handler);
        }
        join_all(sps).await;
    }


}
