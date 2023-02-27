use actix_web_lab::web::spa;
use actix_session::{storage::RedisSessionStore, config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{self, Key},
    web, App, HttpServer,
};
use oauth2::{basic::BasicClient, RevocationUrl};
use oauth2::{
    AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};

use backend::configuration::get_configuration;
use backend::YogaAppData;
use secrecy::Secret;
use tracing_actix_web::TracingLogger;
use sqlx::{postgres::{PgConnectOptions, PgSslMode}, ConnectOptions};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let client = BasicClient::new(
        ClientId::new(configuration.application.client_id.clone()),
        Some(ClientSecret::new(configuration.application.client_secret.clone())),
        AuthUrl::new(configuration.application.oauth_url).unwrap(),
        Some(TokenUrl::new(configuration.application.token_url).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(configuration.application.oauth_redirect_url.clone()).unwrap())
    .set_revocation_uri(RevocationUrl::new(configuration.application.revoke_url).unwrap());

    let yoga_data = web::Data::new(YogaAppData {
        oauth_client: client,
        host: configuration.application.host.clone(),
        port: configuration.application.port.clone(),
        client_id: Secret::new(configuration.application.client_id.clone()),
        client_secret: Secret::new(configuration.application.client_secret.clone()),
        oauth_redirect_url: configuration.application.oauth_redirect_url,
        after_login_url: configuration.application.after_login_url,
    });

    //let redis_uri = "redis://127.0.0.1:6379";
    //let redis_store = RedisSessionStore::new(redis_uri).await.unwrap();

    let bind_address = (configuration.application.host, configuration.application.port.parse::<u16>().unwrap());

    tracing::info!("serving yogamat backend at https://{}:{}", bind_address.0, bind_address.1);

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(backend::routes::oauth::request_login_uri)
            .service(backend::routes::oauth::oauth_login_redirect)
            .route("/logout", web::get().to(backend::routes::oauth::logout))
            .route("/health_check", web::get().to(backend::routes::health_check))
            .route("/poses", web::get().to(backend::routes::poses::look_at_poses))
            .service(
                spa()
                .index_file("./dist/index.html")
                .static_resources_mount("/")
                .static_resources_location("./dist")
                .finish()
            )
            .app_data(yoga_data.clone())
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
    })
    .bind(bind_address)?
    .run()
    .await
}
