use std::net::SocketAddr;
use std::sync::Arc;
use hyper::{Request, Response, StatusCode, Error};
use hyper::server::{Service, Http};
use hyper::header::{ContentLength};
use futures::{Future, Stream};
use serde_json::{self, Value};

macro_rules! return_error {
    ( $status_code:expr, $error:expr ) => {{
        let error: &[u8] = $error.as_bytes();
        return Response::new()
            .with_status($status_code)
            .with_header(ContentLength(error.len() as u64))
            .with_body(error);
    }};
}

pub(super) struct SlackAppServerState {
    verif_token: String,
}

pub struct SlackAppServer {
    state: Arc<SlackAppServerState>,
}

impl SlackAppServer {
    pub fn start(listen_addr: SocketAddr, verif_token: String) {
        if verif_token.is_empty() {
            panic!("Slack app verification token must not be empty, check the server config");
        }

        let state = Arc::new(SlackAppServerState{
           verif_token,
        });

        let server = Http::new().bind(&listen_addr, move || Ok(SlackAppServer{
            state: state.clone(),
        })).unwrap();
        server.run().unwrap();
    }
}

impl Service for SlackAppServer {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let state = self.state.clone();

        Box::new(req.body().concat2().map(move |b| {
            let json: Value = match serde_json::from_slice(b.as_ref()) {
                Ok(json) => json,
                _ => return_error!(StatusCode::BadRequest, "Invalid JSON in request"),
            };

            let maybe_token = json.get("token");
            if maybe_token.is_none() || !maybe_token.unwrap().is_string()
                || maybe_token.unwrap().as_str().unwrap() != state.verif_token {
                return_error!(StatusCode::Forbidden, "Invalid or missing verification token");
            }

            let event_type = match json.get("type") {
                Some(v) if v.is_string() => v.as_str().unwrap(),
                _ => return_error!(StatusCode::BadRequest, "Missing or invalid type field in request"),
            };

            match event_type {
                "url_verification" => Self::handle_url_verification(state, &json),
                "event_callback" => Self::handle_event_callback(state, &json),
                _ => {
                    println!("Received unhandled event: {}", json);
                    Response::new()
                },
            }
        }))
    }
}
