static API_BASE: &str = "https://slack.com/api/";

use hyper::header::HeaderValue;
use reqwest::header::HeaderMap;
use reqwest::{header, Client, ClientBuilder};
use serde::ser::Serialize;
use serde_json::Value;
use std::error::Error;

pub struct SlackHttpClient {
    client: Client,
}

impl SlackHttpClient {
    pub fn new(auth_token: &str) -> SlackHttpClient {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&("Bearer ".to_string() + auth_token)).unwrap(),
        );

        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();
        SlackHttpClient { client }
    }

    pub async fn api_call<T: Serialize + ?Sized>(
        &self,
        endpoint: &str,
        params: &T,
    ) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let res = self
            .client
            .post(&(API_BASE.to_string() + endpoint))
            .form(params)
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(From::from(
                "Request failed with status ".to_owned() + &res.status().to_string(),
            ));
        }
        let json: Value = res.json().await?;
        let obj = match json.as_object() {
            Some(obj) => obj.to_owned(),
            _ => return Err(From::from("JSON response is not an object")),
        };
        if let Some(err) = obj.get("error") {
            return Err(From::from(
                "Request failed with error ".to_owned() + &err.to_string(),
            ));
        }
        return Ok(json);
    }
}
