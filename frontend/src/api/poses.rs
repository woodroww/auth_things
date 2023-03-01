use gloo_console::log;
use reqwasm::http::Request;
//use serde_json::json;
use serde::{Deserialize, Serialize};
use super::errors::ApiError;
use crate::BACKEND_BASE_URL;

// also defined in /Users/matt/prog/rust/bevy_things/yoga_matt/backend/src/routes/poses.rs
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PoseInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct PoseListResponse {
    pub poses: Vec<PoseInfo>,
}

pub async fn get_poses(token: &str) -> Result<PoseListResponse, ApiError> {
    log!("begin get_tasks request");
    let response = Request::new(&format!("{}/poses", BACKEND_BASE_URL))
        .method(reqwasm::http::Method::GET)
        //.header("x-auth-token", token)
        .send()
        .await
        .unwrap();
    if response.ok() {
        Ok(response.json::<PoseListResponse>().await.unwrap())
    } else {
        match response.status() {
            401 => Err(ApiError::NotAuthenticated),
            _ => Err(ApiError::Unknown),
        }
    }
}
