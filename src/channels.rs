use rirc_server;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

// Max number of IRC messages waiting to get ACK'd before we start removing the older oness
const MAX_MSGS_FROM_IRC_BUFFER: usize = 64;

lazy_static! {
    static ref GLOBAL_CHANNELS: RwLock<HashMap<String, Arc<RwLock<rirc_server::Channel>>>>
                        = RwLock::new(HashMap::new());
    static ref GLOBAL_CHANNELS_MSGS_FROM_IRC: RwLock<HashMap<String, RwLock<Vec<String>>>>
                        = RwLock::new(HashMap::new());
    static ref GLOBAL_CHANNELS_ID: RwLock<HashMap<String, String>>
                        = RwLock::new(HashMap::new());
}

pub fn register_channel(slack_channel_id: String, channel: Arc<RwLock<rirc_server::Channel>>) {
    {
        let channel_guard = channel.read().expect("Channel read lock");
        let mut channel_ids_guard = GLOBAL_CHANNELS_ID.write().expect("Channel IDs write lock");
        channel_ids_guard.insert(channel_guard.name.clone(), slack_channel_id.clone());
    }

    {
        let mut channel_msgs_guard = GLOBAL_CHANNELS_MSGS_FROM_IRC.write().expect("Channel IRC msgs write lock");
        channel_msgs_guard.insert(slack_channel_id.clone(), RwLock::new(Vec::new()));
    }

    {
        let mut channels_guard = GLOBAL_CHANNELS.write().expect("Channels write lock");
        channels_guard.insert(slack_channel_id, channel);
    }
}

pub fn get_channel_id(irc_channel_name: &str) -> Option<String> {
    let channel_ids_guard = GLOBAL_CHANNELS_ID.read().expect("Channels read lock");
    channel_ids_guard.get(irc_channel_name).map(|a| a.clone())
}

pub fn get_irc_channel(slack_channel_id: &str) -> Option<Arc<RwLock<rirc_server::Channel>>> {
    let channels_guard = GLOBAL_CHANNELS.read().expect("Channels read lock");
    channels_guard.get(slack_channel_id).map(|a| a.clone())
}

pub fn mark_message_from_irc(slack_channel_id: &str, msg_ts: String) {
    let channel_msgs_guard = GLOBAL_CHANNELS_MSGS_FROM_IRC.read().expect("Channels IRC msgs map read lock");
    let mut msgs_guard = channel_msgs_guard.get(slack_channel_id).unwrap().write().unwrap();
    msgs_guard.push(msg_ts);

    if msgs_guard.len() > MAX_MSGS_FROM_IRC_BUFFER {
        msgs_guard.remove(0);
    }
}

// Returns true if the message was really sent from IRC
pub fn ack_message_from_irc(slack_channel_id: &str, msg_ts: &str) -> bool {
    let channel_msgs_guard = GLOBAL_CHANNELS_MSGS_FROM_IRC.read().expect("Channels IRC msgs map read lock");
    let mut msgs_guard = channel_msgs_guard.get(slack_channel_id).unwrap().write().unwrap();

    let pos = match msgs_guard.iter().position(|x| *x == *msg_ts) {
        Some(x) => x,
        None => return false,
    };
    msgs_guard.remove(pos);
    true
}