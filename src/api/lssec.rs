use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

enum Order {
    Buy,
    Sell,
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
        }
    }

    async fn get_access_token(&self) -> Result<String, anyhow::Error> {
        if let Some(token) = self.cache.get("access_token").await {
            return Ok(token);
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
            .json::<serde_json::Value>()
            .await?;

        match result.get("access_token") {
            Some(token) => {
                self.cache
                    .insert("access_token".to_string(), token.to_string())
                    .await;
                Ok(token.as_str().unwrap().to_string())
            }
            None => Err(anyhow::anyhow!("No access token")),
        }
    }

    async fn get_balance(&self) -> Result<i64, anyhow::Error> {
        let token = self.get_access_token().await.unwrap();
        let mut lmap = reqwest::header::HeaderMap::new();
        lmap.insert(
            "Authorization",
            format!("Bearer {}", token.to_string()).parse().unwrap(),
        );
        lmap.insert("tr_cd", "CSPAQ12200".parse().unwrap());
        lmap.insert("tr_cont", "N".parse().unwrap());
        let result = Client::new()
            .post("https://openapi.ls-sec.co.kr:8080/stock/accno")
            .headers(lmap)
            .json(&serde_json::json!({
              "CSPAQ12200InBlock": {
                "RecCnt": 1,
                "MgmtBrnNo": "1",
                "BalCreTp": "1"
              }
            }))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        let balance = result
            .get("CSPAQ12200OutBlock2")
            .expect("not exist block1")
            .get("MnyOrdAbleAmt")
            .expect("not exist mny ord able amt");
        match balance.as_i64() {
            Some(b) => Ok(b),
            None => Err(anyhow::anyhow!("No balance")),
        }
    }

    async fn get_positions(&self) -> Option<Vec<Position>> {
        let token = self.get_access_token().await.unwrap();
        let mut lmap = reqwest::header::HeaderMap::new();
        lmap.insert(
            "Authorization",
            // format!("Bearer {}", token.to_string()).parse().unwrap(),
            format!("Bearer {}", "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJ0b2tlbiIsImF1ZCI6IjM0YTAxZWEyLTI1MGItNDhjMi04YjY4LWE2MzVhNzYzMTk0MCIsIm5iZiI6MTcxOTg4MDI5MSwiZ3JhbnRfdHlwZSI6IkNsaWVudCIsImlzcyI6InVub2d3IiwiZXhwIjoxNzE5OTU3NTk5LCJpYXQiOjE3MTk4ODAyOTEsImp0aSI6IlBTNDVoSUZ3MVh1N2FwemlMUWRVYzRqTkxheklQYWNRZHFjWCJ9.BQppSJpH7UF04J7WD7Ab82nSXdpnJhxVa15Ssh6OqzzmNIEdANFkCdMOSYQnOVfSY_TYsinYChKfQS4oDWn14A").parse().unwrap(),
        );
        lmap.insert("tr_cd", "t0424".parse().unwrap());
        lmap.insert("tr_cont", "N".parse().unwrap());
        let result = Client::new()
            .post("https://openapi.ls-sec.co.kr:8080/stock/accno")
            .headers(lmap)
            .json(&serde_json::json!({
              "t0424InBlock": {
                "prcgb": "",
                "chegb": "",
                "dangb": "",
                "charge": "",
                "cts_expcode": ""
              }
            }))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        // println!("{:?}", result);
        // result.get("t0424OutBlock1").expect("not exist block1").as_array().unwrap().iter().for_each(|x| {
        //     let position = serde_json::from_value::<Position>(x.clone()).unwrap();
        //     println!("{:?}", position);
        // });

        // result.get("t0520OutBlock1").expect("not exist block1").as_array().expect("does not array value").iter().map(|x| {
        //     match serde_json::from_value::<Position>(x.clone()).map_err(|e| e.into()) {
        //         Ok(p) => p,
        //     }
        // }).collect::<Result<Vec<Position>, _>>().unwrap();

        let positions: Vec<Position> = result
            .get("t0424OutBlock1")?
            // .expect("not exist block1")
            .as_array()?
            // .expect("does not array value")
            .iter()
            .map(|x| serde_json::from_value(x.clone()).expect("Failed to deserialize"))
            .collect();

        Some(positions)
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
        client.get_positions().await;
    }

    #[tokio::test]
    async fn test_get_balance() {
        let client = LsSecClient::new(KEY.to_string(), SECRET.to_string());
        let balance = client.get_balance().await;
        println!("{:?}", balance)
    }
}
