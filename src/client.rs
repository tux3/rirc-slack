use crate::slack::Slack;
use crate::settings::GLOBAL_SETTINGS;
use crate::settings::UserProfile;
use crate::channels::{register_channel, get_channel_id, mark_message_from_irc};
use crate::users::register_username;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::error::Error;
use rirc_server::{ServerCallbacks, Client as IRCClient, Channel as IRCChannel, Message as IRCMsg};
use futures::executor::block_on;

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

fn on_client_registering(irc_client: &mut IRCClient) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let nick = match irc_client.get_nick() {
        Some(nick) => nick,
        _ => return Err(From::from("Slack gateway couldn't determine your nick!")),
    };
    let settings = GLOBAL_SETTINGS.read().unwrap();
    let profile = match settings.user_profiles.get(&nick) {
        Some(profile) => profile,
        _ => return Err(From::from("Your nick is not registered with the Slack gateway!")),
    };

    println!("Registering: {} ({})", irc_client.addr, nick);

    let mut clients = GLOBAL_CLIENTS.write().unwrap();
    clients.insert(irc_client.addr.to_owned(), Client::new(irc_client, profile));

    Ok(true)
}

fn on_client_registered(irc_client: &IRCClient) -> Result<(), Box<dyn Error + Send + Sync>> {
    let clients = GLOBAL_CLIENTS.read().unwrap();
    let client = match clients.get(&irc_client.addr) {
        Some(client) => client,
        _ => return Err(From::from("Client just registered, but isn't in our list!")),
    };

    let slack = client.slack.clone();
    tokio::spawn(async move {
        if let Ok(users_list) = slack.users_list().await {
            users_list.into_iter().for_each(|user_info| {
                register_username(user_info.id, user_info.name);
            });
        }
    });

    block_on(client.slack.channels_list())?.into_iter()
        .filter(|c| c.is_member)
        .for_each(|channel| {
            let irc_chan_name = "#".to_owned() + &channel.name;
            let _ = block_on(irc_client.join(&irc_chan_name));
            let client_channels_guard = block_on(irc_client.channels.read());
            if let Some(irc_chan) = client_channels_guard.get(&irc_chan_name.to_ascii_uppercase()) {
                block_on(register_channel(channel.id.clone(), irc_chan.upgrade().unwrap()));
            }
        });

    Ok(())
}

fn on_client_disconnect(addr: &SocketAddr) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Disconnected: {}", addr);
    let mut clients = GLOBAL_CLIENTS.write().unwrap();
    clients.remove(addr);

    Ok(())
}

fn on_client_channel_message(client: &IRCClient, chan: &IRCChannel, msg: &IRCMsg) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let msg_text = msg.params.iter().skip(1).map(|s| &**s).collect::<Vec<&str>>().join(" ");

    let channel_id = match block_on(get_channel_id(&chan.name)) {
        Some(channel_id) => channel_id,
        None => return Err(From::from("Couldn't find matching Slack channel for IRC message")),
    };

    let clients = GLOBAL_CLIENTS.read().unwrap();
    let client = match clients.get(&client.addr) {
        Some(client) => client,
        _ => return Err(From::from("Client sent message, but isn't in our list!")),
    };

    let msg_ts = block_on(client.slack.post_message(&channel_id, &msg_text))?;

    // Mark the message we just sent as coming from IRC, so we ignore it when Slack sends it back
    block_on(mark_message_from_irc(&channel_id, msg_ts));
    Ok(true)
}

pub fn get_server_callbacks() -> ServerCallbacks {
    ServerCallbacks {
        on_client_registering,
        on_client_registered,
        on_client_disconnect,
        on_client_channel_message,
        ..Default::default()
    }
}
