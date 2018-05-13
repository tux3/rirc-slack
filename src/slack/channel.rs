
#[derive(Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub creator: String,
    pub created: u64,
    pub is_member: bool,
    pub is_channel: bool,
    pub is_archived: bool,
}
