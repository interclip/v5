use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct APIResponse {
    pub status: APIStatus,
    pub result: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum APIStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "error")]
    Error,
}
