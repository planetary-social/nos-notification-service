use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationEventContent {
    pub apns_token: String,
    pub relays: Vec<String>,
    pub locale: String,
}
