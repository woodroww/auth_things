use gloo_console::log;
use reqwasm::http::Request;
//use serde_json::json;
use serde::{Deserialize, Serialize};
use super::errors::ApiError;
use crate::API_BASE_URL;

// also defined in /Users/matt/prog/rust/bevy_things/yoga_matt/backend/src/routes/poses.rs
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct PoseInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PoseListResponse {
    pub poses: Vec<PoseInfo>,
}

pub async fn get_poses(token: &str) -> Result<PoseListResponse, ApiError> {
    log!("begin get_tasks request");
    let response = Request::new(&format!("{}/poses", API_BASE_URL))
        .method(reqwasm::http::Method::GET)
        .header("x-auth-token", token)
        .send()
        .await;
    match response {
        Ok(response) => {
            log!("get_poses reqwasm ok");
            if response.ok() {
                let json_response = response.json::<PoseListResponse>().await.unwrap();
                log!(format!("get_poses response text {:?}", json_response));
                return Ok(json_response);
            }
        }
        Err(_) => log!("get_poses reqwasm err"),
    }
    Err(ApiError::Unknown)
}
