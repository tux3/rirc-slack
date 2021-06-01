use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use serde_json::{self, Value};
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;

macro_rules! return_error {
    ( $status_code:expr, $error:expr ) => {{
        let error: &[u8] = $error.as_bytes();
        return Response::builder()
            .status($status_code)
            .header("Content-Length", (error.len() as u64))
            .body(error.into())
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>);
    }};
}

mod handlers;

pub(super) struct SlackAppServerState {
    verif_token: String,
}

pub struct SlackAppServer;

impl SlackAppServer {
    pub async fn start(listen_addr: SocketAddr, verif_token: String) {
        if verif_token.is_empty() {
            panic!("Slack app verification token must not be empty, check the server config");
        }

        let state: &'static SlackAppServerState =
            Box::leak(Box::new(SlackAppServerState { verif_token }));

        let service = make_service_fn(|_| async move {
            let service_handler = move |req| Self::slack_service(state, req);
            Ok::<_, hyper::Error>(service_fn(service_handler))
        });
        let hyper_server = Server::bind(&listen_addr).serve(service);

        hyper_server.await.unwrap();
    }

    async fn try_process_request(
        state: &SlackAppServerState,
        req: Request<Body>,
    ) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
        let payload = hyper::body::to_bytes(req.into_body()).await?;

        let json: Value = match serde_json::from_slice(payload.as_ref()) {
            Ok(json) => json,
            _ => return_error!(StatusCode::BAD_REQUEST, "Invalid JSON in request"),
        };

        let maybe_token = json.get("token");
        if maybe_token.is_none()
            || !maybe_token.unwrap().is_string()
            || maybe_token.unwrap().as_str().unwrap() != state.verif_token
        {
            return_error!(
                StatusCode::FORBIDDEN,
                "Invalid or missing verification token"
            );
        }

        let event_type = match json.get("type") {
            Some(v) if v.is_string() => v.as_str().unwrap(),
            _ => return_error!(
                StatusCode::BAD_REQUEST,
                "Missing or invalid type field in request"
            ),
        };

        match event_type {
            "url_verification" => Self::handle_url_verification(state, &json).await,
            "event_callback" => Self::handle_event_callback(state, &json).await,
            _ => Err(format!("Received unhandled event: {}", json).into()),
        }
    }

    async fn slack_service(
        state: &SlackAppServerState,
        req: Request<Body>,
    ) -> Result<Response<Body>, Infallible> {
        match Self::try_process_request(state, req).await {
            Ok(reply) => Ok(reply),
            Err(err) => {
                println!("Error processing slack request: {}", err);
                Ok(Response::new("".into()))
            }
        }
    }
}
