use crate::session_state::TypedSession;
use actix_web::{web, HttpResponse, http::header::ContentType};

pub async fn look_at_poses(
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    let authorized = match session.get_access_token() {
        Ok(maybe_token) => {
            match maybe_token {
                Some(_) => {
                    println!("we have a token");
                    true
                }
                None => {
                    println!("no token");
                    false
                }
            }
        }
        Err(err) => {
            println!("error from session {}", err);
            false
        }
    };
    if authorized {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(format!(
                r#"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Poses</title>
    </head>
    <body>
    <p>Welcome to yogamat my friendly registered user</p>
    </body>
    </html>"#,
            )))
    } else {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(format!(
                r#"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Poses</title>
    </head>
    <body>
    <p>You can't come in here this is for true yogis only!</p>
    </body>
    </html>"#,
            )))
    }
}
