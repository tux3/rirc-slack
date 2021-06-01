use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Max number of IRC messages waiting to get ACK'd before we start removing the older oness
const MAX_MSGS_FROM_IRC_BUFFER: usize = 64;

lazy_static! {
    static ref GLOBAL_CHANNELS: RwLock<HashMap<String, Arc<RwLock<rirc_server::Channel>>>> =
        RwLock::new(HashMap::new());
    static ref GLOBAL_CHANNELS_MSGS_FROM_IRC: RwLock<HashMap<String, RwLock<Vec<String>>>> =
        RwLock::new(HashMap::new());
    static ref GLOBAL_CHANNELS_ID: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub async fn register_channel(
    slack_channel_id: String,
    channel: Arc<RwLock<rirc_server::Channel>>,
) {
    {
        let channel_guard = channel.read().await;
        let mut channel_ids_guard = GLOBAL_CHANNELS_ID.write().await;
        channel_ids_guard.insert(channel_guard.name.clone(), slack_channel_id.clone());
    }

    {
        let mut channel_msgs_guard = GLOBAL_CHANNELS_MSGS_FROM_IRC.write().await;
        channel_msgs_guard.insert(slack_channel_id.clone(), RwLock::new(Vec::new()));
    }

    {
        let mut channels_guard = GLOBAL_CHANNELS.write().await;
        channels_guard.insert(slack_channel_id, channel);
    }
}

pub async fn get_channel_id(irc_channel_name: &str) -> Option<String> {
    let channel_ids_guard = GLOBAL_CHANNELS_ID.read().await;
    channel_ids_guard.get(irc_channel_name).cloned()
}

pub async fn get_irc_channel(slack_channel_id: &str) -> Option<Arc<RwLock<rirc_server::Channel>>> {
    let channels_guard = GLOBAL_CHANNELS.read().await;
    channels_guard.get(slack_channel_id).cloned()
}

pub async fn mark_message_from_irc(slack_channel_id: &str, msg_ts: String) {
    let channel_msgs_guard = GLOBAL_CHANNELS_MSGS_FROM_IRC.read().await;
    let mut msgs_guard = channel_msgs_guard
        .get(slack_channel_id)
        .unwrap()
        .write()
        .await;
    msgs_guard.push(msg_ts);

    if msgs_guard.len() > MAX_MSGS_FROM_IRC_BUFFER {
        msgs_guard.remove(0);
    }
}

// Returns true if the message was really sent from IRC
pub async fn ack_message_from_irc(slack_channel_id: &str, msg_ts: &str) -> bool {
    let channel_msgs_guard = GLOBAL_CHANNELS_MSGS_FROM_IRC.read().await;
    let mut msgs_guard = channel_msgs_guard
        .get(slack_channel_id)
        .unwrap()
        .write()
        .await;

    let pos = match msgs_guard.iter().position(|x| *x == *msg_ts) {
        Some(x) => x,
        None => return false,
    };
    msgs_guard.remove(pos);
    true
}
