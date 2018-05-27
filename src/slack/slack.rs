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
    pub fn test_request(&self) -> Result<(), Box<Error>> {
        self.http_client.api_call::<[(&str, &str)]>("api.test", &[])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn test_auth(&self) -> Result<(), Box<Error>> {
        let params = [("token", &self.token)];
        self.http_client.api_call("auth.test", &params)?;
        Ok(())
    }

    pub fn post_message(&self, channel: &str, message: &str) -> Result<(String), Box<Error>> {
        let params = [
            ("channel", Value::from(channel)),
            ("text", Value::from(message)),
            ("as_user", Value::from(true)),
        ];
        let mut json = self.http_client.api_call("chat.postMessage", &params)?;
        Ok(serde_json::from_value(json["ts"].take()).unwrap())
    }

    // NOTE: Won't return archived channels
    pub fn channels_list(&self) -> Result<Vec<Channel>, Box<Error>> {
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
            let mut json = self.http_client.api_call("channels.list", &params)?;
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

    pub fn users_list(&self) -> Result<Vec<UserInfo>, Box<Error>> {
        let mut result = Vec::new();
        let mut next_cursor = None;

        loop {
            let params = if let Some(cursor) = next_cursor {
                [("limit", Value::from(500)), ("cursor", Value::from(cursor))]
            } else {
                [("limit", Value::from(500)), ("cursor", Value::from(""))]
            };
            let mut json = self.http_client.api_call("users.list", &params)?;
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
