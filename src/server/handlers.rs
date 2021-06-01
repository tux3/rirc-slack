use crate::channels::{ack_message_from_irc, get_irc_channel};
use crate::server::{server::SlackAppServerState, SlackAppServer};
use crate::users::get_username;
use hyper::{Body, Response, StatusCode};
use rirc_server::Message;
use serde_json::{Map, Value};
use std::error::Error;

impl SlackAppServer {
    pub(super) async fn handle_url_verification(
        _state: &SlackAppServerState,
        json: &Value,
    ) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        let challenge = match json.get("challenge") {
            Some(v) if v.is_string() => v.as_str().unwrap().to_owned(),
            _ => return_error!(
                StatusCode::BAD_REQUEST,
                "Missing or invalid challenge field in request"
            ),
        };

        Ok(Response::builder()
            .header("Content-Length", challenge.len() as u64)
            .body(challenge.into())
            .unwrap())
    }

    pub(super) async fn handle_event_callback(
        state: &SlackAppServerState,
        json: &Value,
    ) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        let event_object = match json.get("event") {
            Some(v) if v.is_object() => v.as_object().unwrap(),
            _ => return_error!(
                StatusCode::BAD_REQUEST,
                "Missing or invalid event field in request"
            ),
        };
        let event_type = match event_object.get("type") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(
                StatusCode::BAD_REQUEST,
                "Missing or invalid event callback type field in request"
            ),
        };

        match event_type {
            "message" => Self::handle_message_event_callback(state, event_object).await,
            _ => {
                println!("Received unhandled event callback: {}", json);
                Ok(Response::new("".into()))
            }
        }
    }

    pub(super) async fn handle_message_event_callback(
        _state: &SlackAppServerState,
        event_object: &Map<String, Value>,
    ) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        if event_object.contains_key("subtype") {
            println!(
                "Received unhandled message event with subtype: {:?}",
                event_object
            );
            return Ok(Response::new("".into()));
        }

        let channel = match event_object.get("channel") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(
                StatusCode::BAD_REQUEST,
                "Missing or invalid channel field in message event"
            ),
        };

        let user = match event_object.get("user") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(
                StatusCode::BAD_REQUEST,
                "Missing or invalid user field in message event"
            ),
        };

        let text = match event_object.get("text") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(
                StatusCode::BAD_REQUEST,
                "Missing or invalid text field in message event"
            ),
        };

        let ts = match event_object.get("ts") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(
                StatusCode::BAD_REQUEST,
                "Missing or invalid ts field in message event"
            ),
        };

        if ack_message_from_irc(&channel, ts).await {
            // If the message comes from IRC, whoever's connected to the server already received it!
            return Ok(Response::new("".into()));
        }

        println!(
            "Received message '{}' ts {} from {} in channel {}",
            text, ts, user, channel
        );

        let username = get_username(user).unwrap_or(user.to_owned());

        if let Some(channel) = get_irc_channel(&channel).await {
            let text = text.to_owned();
            tokio::spawn(async move {
                let channel_guard = channel.write().await;
                let _ = channel_guard
                    .send(
                        Message {
                            tags: Vec::new(),
                            source: Some(username.clone() + "!~" + &username + "@slack.com"),
                            command: "PRIVMSG".to_owned(),
                            params: vec![channel_guard.name.to_owned(), text],
                        },
                        None,
                    )
                    .await;
            });
        }

        Ok(Response::new("".into()))
    }
}
