use server::{SlackAppServer, server::SlackAppServerState};
use serde_json::{Map, Value};
use std::sync::{Arc};
use hyper::{Response, StatusCode};
use hyper::header::{ContentLength};
use channels::get_channel;
use users::get_username;
use rirc_server::{Message};
use futures::Future;

impl SlackAppServer {
    pub(super) fn handle_url_verification(state: Arc<SlackAppServerState>, json: &Value) -> Response {
        let challenge = match json.get("challenge") {
            Some(v) if v.is_string() => v.as_str().unwrap().to_owned(),
            _ => return_error!(StatusCode::BadRequest, "Missing or invalid challenge field in request"),
        };

        Response::new().with_header(ContentLength(challenge.len() as u64)).with_body(challenge)
    }

    pub(super) fn handle_event_callback(state: Arc<SlackAppServerState>, json: &Value) -> Response {
        let event_object = match json.get("event") {
            Some(v) if v.is_object() => v.as_object().unwrap(),
            _ => return_error!(StatusCode::BadRequest, "Missing or invalid event field in request"),
        };
        let event_type = match event_object.get("type") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(StatusCode::BadRequest, "Missing or invalid event callback type field in request"),
        };

        match event_type {
            "message" => Self::handle_message_event_callback(state, event_object),
            _ => {
                println!("Received unhandled event callback: {}", json);
                Response::new()
            },
        }
    }

    pub(super) fn handle_message_event_callback(state: Arc<SlackAppServerState>, event_object: &Map<String, Value>) -> Response {
        if event_object.contains_key("subtype") {
            println!("Received unhandled message event with subtype: {:?}", event_object);
            return Response::new();
        }

        let channel = match event_object.get("channel") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(StatusCode::BadRequest, "Missing or invalid channel field in message event"),
        };

        let user = match event_object.get("user") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(StatusCode::BadRequest, "Missing or invalid user field in message event"),
        };

        let text = match event_object.get("text") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(StatusCode::BadRequest, "Missing or invalid text field in message event"),
        };

        println!("Received message '{}' from {} in channel {}", text, user, channel);

        let username = get_username(user).unwrap_or(user.to_owned());

        if let Some(channel) = get_channel(&channel) {
            let channel_guard = channel.write().expect("Channel write lock broken!");
            // FIXME: We're supposed to reply to Slack under 3s, so waiting on the send to each client of the channel is a bit counterproductive...
            channel_guard.send(Message {
                tags: Vec::new(),
                source: Some(username.clone()+"!~"+&username+"@slack.com"),
                command: "PRIVMSG".to_owned(),
                params: vec!(channel_guard.name.to_owned(), text.to_owned()),
            }, None).wait().ok();
        }

        Response::new()
    }
}