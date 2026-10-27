#[derive(Clone)]
pub struct PlayerProfile {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub client_id: Option<String>,
    pub x_uid: Option<String>,
    pub online: bool,
}

impl PlayerProfile {
    pub fn new() -> Self {
        Self {
            username: "".to_string(),
            uuid: "".to_string(),
            access_token: "".to_string(),
            client_id: None,
            x_uid: None,
            online: false,
        }
    }
    
    pub fn offline(username: String) -> PlayerProfile {
        let uuid = crate::auth::generate_offline_uuid(username.clone());
        PlayerProfile {
            username,
            uuid,
            access_token: "offline".to_string(),
            client_id: None,
            x_uid: None,
            online: false,
        }
    }
}