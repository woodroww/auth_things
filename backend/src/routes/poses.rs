use crate::session_state::TypedSession;
use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

// also defined in /Users/matt/prog/rust/bevy_things/yoga_matt/frontend/src/api/poses.rs
#[derive(Serialize, Deserialize)]
pub struct PoseInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct PoseListResponse {
    poses: Vec<PoseInfo>,
}

#[actix_web::get("/poses")]
pub async fn look_at_poses(
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    match session.get_access_token() {
        Ok(maybe_token) => {
            match maybe_token {
                Some(_) => {
                    let poses = vec![
                        PoseInfo { id: 0, name: "updog".to_string() },
                        PoseInfo { id: 1, name: "downdog".to_string() },
                        PoseInfo { id: 2, name: "yoganidrasana".to_string() },
                    ];
                    tracing::info!("we have a token");
                    Ok(HttpResponse::Ok().json(PoseListResponse { poses }))
                }
                None => {
                    tracing::info!("no token");
                    Ok(HttpResponse::Unauthorized().into())
                }
            }
        }
        Err(err) => {
            tracing::info!("error getting access token from session {}", err);
            Ok(HttpResponse::InternalServerError().into())
        }
    }
}
