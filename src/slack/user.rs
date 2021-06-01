#[derive(Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub team_id: String,
    pub name: String,
    pub real_name: Option<String>,
    pub is_bot: bool,
    pub is_app_user: bool,
    pub deleted: bool,
}
