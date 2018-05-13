use rirc_server;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

lazy_static! {
    static ref GLOBAL_CHANNELS: RwLock<HashMap<String, Arc<RwLock<rirc_server::Channel>>>>
                        = RwLock::new(HashMap::new());
}

pub fn register_channel(slack_channel_id: String, channel: Arc<RwLock<rirc_server::Channel>>) {
    let mut channels_guard = GLOBAL_CHANNELS.write().expect("Channels write lock");
    channels_guard.insert(slack_channel_id, channel);
}

pub fn get_channel(slack_channel_id: &str) -> Option<Arc<RwLock<rirc_server::Channel>>> {
    let channels_guard = GLOBAL_CHANNELS.read().expect("Channels read lock");
    channels_guard.get(slack_channel_id).map(|a| a.clone())
}