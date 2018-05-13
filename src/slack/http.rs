static API_BASE: &str = "https://slack.com/api/";

use reqwest::{Client, ClientBuilder, header};
use serde::ser::Serialize;
use serde_json::{Value};
use std::error::Error;

pub struct SlackHttpClient {
    client: Client,
}

impl SlackHttpClient {
    pub fn new(auth_token: &str) -> SlackHttpClient {
        let mut headers = header::Headers::new();
        headers.set(header::Authorization("Bearer ".to_string()+auth_token));

        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();
        SlackHttpClient{
            client,
        }
    }

    pub fn api_call<T: Serialize + ?Sized>(&self, endpoint: &str, params: &T) -> Result<Value, Box<Error>> {
        let mut res = self.client.post(&(API_BASE.to_string()+endpoint))
            .form(params)
            .send()?;
        if !res.status().is_success() {
            return Err(From::from("Request failed with status ".to_owned()+&res.status().to_string()))
        }
        let json: Value = res.json()?;
        let obj = match json.as_object() {
            Some(obj) => obj.to_owned(),
            _ => return Err(From::from("JSON response is not an object")),
        };
        if let Some(err) = obj.get("error") {
            return Err(From::from("Request failed with error ".to_owned()+&err.to_string()))
        }
        return Ok(json)
    }
}