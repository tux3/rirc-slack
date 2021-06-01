use serde_json;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, RwLock};

static CONFIG_FILE_RELPATH: &'static str = ".config/rirc_slack.json";

lazy_static! {
    pub static ref GLOBAL_SETTINGS: Arc<RwLock<Settings>> =
        Arc::new(RwLock::new(read_settings().unwrap_or(Settings::default())));
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserProfile {
    pub name: String,
    pub slack_token: String,
}

#[derive(Serialize, Deserialize)]
struct SettingsFile {
    pub irc_listen_addr: String,
    pub slack_app_listen_addr: String,
    pub slack_app_verif_token: String,
    pub user_profiles: Vec<UserProfile>,
}

pub struct Settings {
    pub irc_listen_addr: String,
    pub slack_app_listen_addr: String,
    pub slack_app_verif_token: String,
    pub user_profiles: HashMap<String, UserProfile>, // Names to profiles
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            irc_listen_addr: "0.0.0.0:8080".to_owned(),
            slack_app_listen_addr: "0.0.0.0:8080".to_owned(),
            slack_app_verif_token: String::new(),
            user_profiles: HashMap::new(),
        }
    }
}

fn get_settings_file_path() -> String {
    let home = env::var("HOME").unwrap();
    home + "/" + CONFIG_FILE_RELPATH
}

fn read_settings() -> Result<Settings, Box<dyn Error>> {
    let mut file: File = File::open(get_settings_file_path())?;
    let contents = &mut String::new();
    file.read_to_string(contents)?;
    let settings_file: SettingsFile = serde_json::from_str(&contents)
        .expect("Error parsing config file, are you missing some settings?");

    let user_profiles = settings_file
        .user_profiles
        .into_iter()
        .map(|u| (u.name.to_string(), u))
        .collect::<HashMap<String, UserProfile>>();

    Ok(Settings {
        irc_listen_addr: settings_file.irc_listen_addr,
        slack_app_listen_addr: settings_file.slack_app_listen_addr,
        slack_app_verif_token: settings_file.slack_app_verif_token,
        user_profiles,
    })
}

#[allow(dead_code)]
pub fn save_settings() -> Result<(), Box<dyn Error>> {
    let settings = GLOBAL_SETTINGS.read()?;

    let mut file = File::create(get_settings_file_path())?;
    let user_profiles = settings
        .user_profiles
        .clone()
        .into_iter()
        .map(|(_, profile)| profile)
        .collect::<Vec<UserProfile>>();

    let settings_file = SettingsFile {
        irc_listen_addr: settings.irc_listen_addr.clone(),
        slack_app_listen_addr: settings.slack_app_listen_addr.clone(),
        slack_app_verif_token: settings.slack_app_verif_token.clone(),
        user_profiles,
    };
    let encoded = serde_json::to_string(&settings_file)?;
    file.set_len(0)?;
    file.write_all(encoded.as_bytes())?;
    file.flush()?;
    Ok(())
}
