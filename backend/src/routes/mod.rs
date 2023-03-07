pub mod oauth;
pub mod poses;

use actix_web::HttpResponse;

#[actix_web::get("/health_check")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

