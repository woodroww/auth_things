use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, SessionMiddleware,
};
use actix_web::{
    cookie::{self, Key},
    web, App, HttpServer, http
};
use actix_cors::Cors;
use actix_web_lab::web::spa;
use oauth2::{basic::BasicClient, RevocationUrl};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

use backend::configuration::{get_configuration, DatabaseSettings};
use backend::YogaAppData;
use secrecy::Secret;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let client = BasicClient::new(
        ClientId::new(configuration.application.client_id.clone()),
        Some(ClientSecret::new(
            configuration.application.client_secret.clone(),
        )),
        AuthUrl::new(configuration.application.oauth_url).unwrap(),
        Some(TokenUrl::new(configuration.application.token_url).unwrap()),
    )
    .set_redirect_uri(
        RedirectUrl::new(configuration.application.oauth_redirect_url.clone()).unwrap(),
    )
    .set_revocation_uri(RevocationUrl::new(configuration.application.revoke_url).unwrap());

    let connection_pool = get_connection_pool(&configuration.database);
    let db_pool = web::Data::new(connection_pool);

    let yoga_data = web::Data::new(YogaAppData {
        oauth_client: client,
        host: configuration.application.host.clone(),
        port: configuration.application.port.clone(),
        client_id: Secret::new(configuration.application.client_id.clone()),
        client_secret: Secret::new(configuration.application.client_secret.clone()),
        oauth_redirect_url: configuration.application.oauth_redirect_url,
        after_login_url: configuration.application.after_login_url,
    });

    let bind_address = (
        configuration.application.host,
        configuration.application.port.parse::<u16>().unwrap(),
    );

    tracing::info!(
        "serving yogamat backend at http://{}:{}",
        bind_address.0,
        bind_address.1
    );

    HttpServer::new(move || {
        let allowed_origins = configuration.application.allowed_origins.clone();
        let cors = Cors::default()
            .allowed_origin_fn(move |origin, _req_head| {
                for allowed in allowed_origins.iter() {
                    if origin.as_ref().starts_with(allowed.as_bytes()) {
                        return true;
                    }
                }
                false
            })
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::ACCESS_CONTROL_ALLOW_HEADERS,
                http::header::ACCESS_CONTROL_ALLOW_METHODS,
                http::header::CONTENT_TYPE,
                http::header::HeaderName::from_lowercase(b"x-auth-token").unwrap(),
            ])
            .allowed_methods(vec!["GET", "POST", "PATCH"])
            .max_age(3600);
        App::new()
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api/v1")
                    .service(backend::routes::oauth::request_login_uri)
                    .service(backend::routes::oauth::oauth_login_redirect)
                    .service(backend::routes::oauth::logout)
                    .service(backend::routes::health_check)
                    .service(backend::routes::poses::look_at_poses)
            )
            .app_data(yoga_data.clone())
            .app_data(db_pool.clone())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    // .cookie_secure(true) is default, cookies set as secure, only transmitted when https
                    // .cookie_http_only(true) is default, no javascript access to cookies
                    // .cookie_content_security(CookieContentSecurity::Private) is default, encrypted but not signed
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::hours(2)),
                    )
                    .build(),
            )
            .wrap(cors)
    })
    .bind(bind_address)?
    .run()
    .await
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
