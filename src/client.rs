use slack::Slack;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::error::Error;
use std::thread;
use rirc_server::{ServerCallbacks, Client as IRCClient};
use settings::GLOBAL_SETTINGS;
use settings::UserProfile;
use channels::register_channel;
use users::register_username;

lazy_static! {
    pub static ref GLOBAL_CLIENTS: Arc<RwLock<HashMap<SocketAddr, Client>>>
                        = Arc::new(RwLock::new(HashMap::new()));
}

pub struct Client {
    pub addr: SocketAddr,
    pub slack: Arc<Slack>,
}

impl Client {
    fn new(irc_client: &IRCClient, profile: &UserProfile) -> Client {
        Client {
            addr: irc_client.addr.to_owned(),
            slack: Arc::new(Slack::new(&profile.slack_token)),
        }
    }
}

fn on_client_registering(irc_client: &mut IRCClient) -> Result<bool, Box<Error>> {
    let nick = match irc_client.get_nick() {
        Some(nick) => nick,
        _ => return Err(From::from("Slack gateway couldn't determine your nick!")),
    };
    let settings = GLOBAL_SETTINGS.read()?;
    let profile = match settings.user_profiles.get(&nick) {
        Some(profile) => profile,
        _ => return Err(From::from("Your nick is not registered with the Slack gateway!")),
    };

    println!("Registering: {} ({})", irc_client.addr, nick);

    let mut clients = GLOBAL_CLIENTS.write()?;
    clients.insert(irc_client.addr.to_owned(), Client::new(irc_client, profile));

    Ok(true)
}

fn on_client_registered(irc_client: &mut IRCClient) -> Result<(), Box<Error>> {
    let clients = GLOBAL_CLIENTS.read()?;
    let client = match clients.get(&irc_client.addr) {
        Some(client) => client,
        _ => return Err(From::from("Client just registered, but isn't in our list!")),
    };

    let slack = client.slack.clone();
    let list_users_thread = thread::spawn(move || {
        if let Ok(users_list) = slack.users_list() {
            users_list.into_iter().for_each(|user_info| {
                register_username(user_info.id, user_info.name);
            });
        }
    });

    client.slack.channels_list()?.into_iter()
        .filter(|c| c.is_member)
        .for_each(|channel| {
            let irc_chan_name = "#".to_owned() + &channel.name;
            irc_client.join(&irc_chan_name);
            let client_channels_guard = irc_client.channels.read().expect("Client channels read lock broken");
            if let Some(irc_chan) = client_channels_guard.get(&irc_chan_name.to_ascii_uppercase()) {
                register_channel(channel.id.clone(), irc_chan.upgrade().unwrap());
            }
        });

    list_users_thread.join().ok();

    Ok(())
}

fn on_client_disconnect(addr: &SocketAddr) -> Result<(), Box<Error>> {
    println!("Disconnected: {}", addr);
    let mut clients = GLOBAL_CLIENTS.write()?;
    clients.remove(addr);

    Ok(())
}

pub fn get_server_callbacks() -> ServerCallbacks {
    ServerCallbacks {
        on_client_registering,
        on_client_registered,
        on_client_disconnect,
        ..Default::default()
    }
}
