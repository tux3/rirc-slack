extern crate reqwest;
extern crate rirc_server;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate futures;
extern crate hyper;

mod channels;
mod client;
mod server;
mod settings;
mod slack;
mod users;

use client::get_server_callbacks;
use rirc_server::{Server, ServerSettings};
use server::SlackAppServer;
use settings::GLOBAL_SETTINGS;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("rIRC Slack gateway");

    let settings = GLOBAL_SETTINGS.read()?;

    let irc_fut = tokio::spawn(async move {
        let mut irc_server = Server::new(
            ServerSettings {
                listen_addr: "0.0.0.0:6697".parse().unwrap(),
                server_name: "rIRC-slack-gateway".to_owned(),
                ..Default::default()
            },
            get_server_callbacks(),
        );

        irc_server.start().await.unwrap();
    });

    let slack_app_listen_addr = settings.slack_app_listen_addr.parse()?;
    let slack_app_verif_token = settings.slack_app_verif_token.clone();
    let slack_fut = tokio::spawn(SlackAppServer::start(
        slack_app_listen_addr,
        slack_app_verif_token,
    ));

    irc_fut.await.unwrap();
    slack_fut.await.unwrap();

    Ok(())
}
