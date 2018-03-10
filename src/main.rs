extern crate rirc_server;

use rirc_server::{Server, ServerSettings};

fn main() {
    let mut server = Server::new(ServerSettings {
        listen_addr: "0.0.0.0:6697".parse().unwrap(),
        server_name: "rIRC-slack-gateway".to_owned(),
    });

    println!("rIRC Slack gateway");

    server.start();
}
