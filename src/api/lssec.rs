#[cfg(test)]
mod test {

    static KEY: &str = "PS45hIFw1Xu7apziLQdUc4jNLazIPacQdqcX";
    static SECRET: &str = "nzWMVzES7uvxUKyK68nmXb2cHHhOOg8o";
    static TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJ0b2tlbiIsImF1ZCI6IjRkNmUyZGJlLTRmYWYtNDVjZC1iZjBiLTZjNDMwYzAwNjFiYyIsIm5iZiI6MTcxOTQ0NjQwNywiZ3JhbnRfdHlwZSI6IkNsaWVudCIsImlzcyI6InVub2d3IiwiZXhwIjoxNzE5NTI1NTk5LCJpYXQiOjE3MTk0NDY0MDcsImp0aSI6IlBTNDVoSUZ3MVh1N2FwemlMUWRVYzRqTkxheklQYWNRZHFjWCJ9.LYBago-xZtYsv7_mzkmvy-yU9ffXsDB78GV3GZsdU3Pp_w4Wdu1rYsHbLZIvXhk-UPtqACTaICbFOGTzZy583Q";
    use super::*;
    #[tokio::test]
    async fn test() {
        let result = reqwest::Client::new()
            .post("	https://openapi.ls-sec.co.kr:8080/oauth2/token")
            .form(&[
                ("grant_type", "client_credentials"),
                ("appkey", KEY),
                ("appsecretkey", SECRET),
                ("scope", "oob"),
            ])
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        println!("{:?}", result.get("access_token"));
        assert_ne!(result.get("access_token"), None);
    }

    #[tokio::test]
    async fn test_get_account() {
        let mut lmap = reqwest::header::HeaderMap::new();
        lmap.insert(
            "Authorization",
            format!("Bearer {}", TOKEN).parse().unwrap(),
        );
        lmap.insert("tr_cd", "CDPCQ04700".parse().unwrap());
        lmap.insert("tr_cont", "N".parse().unwrap());
        let result = reqwest::Client::new()
            .post("https://openapi.ls-sec.co.kr:8080/stock/accno")
            .headers(lmap)
            .json(&serde_json::json!({
                "CDPCQ04700InBlock1": {
                    "RecCnt": 1,
                    "QryTp": "0",
                    "QrySrtDt": "20240627",
                    "QryEndDt": "20240627",
                    "SrtNo": 0,
                    "PdptnCode": "01",
                    "IsuLgclssCode": "00",
                    "IsuNo": "A304360"
                }
            }))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        println!("{:?}", result);
    }
}
