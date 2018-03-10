extern crate rirc_server;

use rirc_server::Server;

fn main() {
    let addr = "0.0.0.0:6697".parse().unwrap();
    let mut server = Server::new(addr);

    println!("rIRC Slack gateway");

    server.start();
}
