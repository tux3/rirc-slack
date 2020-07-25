use super::http::{SlackHttpClient};
use super::{Channel, UserInfo};
use std::vec::Vec;
use std::error::Error;
use serde_json::{self, Value};

pub struct Slack {
    token: String,
    http_client: SlackHttpClient,
}

impl Slack {
    pub fn new(token: &str) -> Slack {
        return Slack {
            token: token.to_owned(),
            http_client: SlackHttpClient::new(token),
        }
    }

    #[allow(dead_code)]
    pub async fn test_request(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.http_client.api_call::<[(&str, &str)]>("api.test", &[]).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn test_auth(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let params = [("token", &self.token)];
        self.http_client.api_call("auth.test", &params).await?;
        Ok(())
    }

    pub async fn post_message(&self, channel: &str, message: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let params = [
            ("channel", Value::from(channel)),
            ("text", Value::from(message)),
            ("as_user", Value::from(true)),
        ];
        let mut json = self.http_client.api_call("chat.postMessage", &params).await?;
        Ok(serde_json::from_value(json["ts"].take()).unwrap())
    }

    // NOTE: Won't return archived channels
    pub async fn channels_list(&self) -> Result<Vec<Channel>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let mut next_cursor = None;

        loop {
            let params = if let Some(cursor) = next_cursor {
                [("exclude_archived", Value::from(true)), ("exclude_members", Value::from(true)),
                    ("limit", Value::from(500)), ("cursor", Value::from(cursor))]
            } else {
                [("exclude_archived", Value::from(true)), ("exclude_members", Value::from(true)),
                    ("limit", Value::from(500)), ("cursor", Value::from(""))]
            };
            let mut json = self.http_client.api_call("channels.list", &params).await?;
            let jchannels = json["channels"].take();
            let mut next_channels = serde_json::from_value(jchannels)?;
            result.append(&mut next_channels);

            if let Some(meta) = json.get("response_metadata") {
                if let Some(Some(cursor)) = meta.get("next_cursor").map(|v| v.as_str()) {
                    if !cursor.is_empty() {
                        next_cursor = Some(cursor.to_owned());
                        continue;
                    }
                }
            }
            break;
        }

        Ok(result)
    }

    pub async fn users_list(&self) -> Result<Vec<UserInfo>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let mut next_cursor = None;

        loop {
            let params = if let Some(cursor) = next_cursor {
                [("limit", Value::from(500)), ("cursor", Value::from(cursor))]
            } else {
                [("limit", Value::from(500)), ("cursor", Value::from(""))]
            };
            let mut json = self.http_client.api_call("users.list", &params).await?;
            let jmembers = json["members"].take();
            let mut next_members = serde_json::from_value(jmembers)?;
            result.append(&mut next_members);

            if let Some(meta) = json.get("response_metadata") {
                if let Some(Some(cursor)) = meta.get("next_cursor").map(|v| v.as_str()) {
                    if !cursor.is_empty() {
                        next_cursor = Some(cursor.to_owned());
                        continue;
                    }
                }
            }
            break;
        }

        Ok(result)
    }
}
