use std::collections::HashMap;
use std::sync::RwLock;

lazy_static! {
    static ref GLOBAL_USERNAMES: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub fn register_username(slack_user_id: String, username: String) {
    let mut users_guard = GLOBAL_USERNAMES.write().expect("Usernames write lock");
    users_guard.insert(slack_user_id, username);
}

pub fn get_username(slack_user_id: &str) -> Option<String> {
    let users_guard = GLOBAL_USERNAMES.read().expect("Usernames read lock");
    users_guard.get(slack_user_id).map(|a| a.clone())
}
