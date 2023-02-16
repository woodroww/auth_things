use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{self, Key},
    web, App, HttpServer,
};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};

use backend::configuration::{get_configuration, YogaAppData};
use secrecy::Secret;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let fusion_uri = format!("http://{}/oauth2/authorize", configuration.application.oauth_server);
    let token_endpoint = format!("http://{}/oauth2/token", configuration.application.oauth_server);

    let redirect_uri = format!(
        "http://{}:{}/oauth-redirect",
        configuration.application.oauth_redirect_host, configuration.application.port
    );

    let client = BasicClient::new(
        ClientId::new(configuration.application.client_id.clone()),
        Some(ClientSecret::new(configuration.application.client_secret)),
        AuthUrl::new(fusion_uri).unwrap(),
        Some(TokenUrl::new(token_endpoint).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap());

    let yoga_data = web::Data::new(YogaAppData {
        oauth_client: client,
        host: configuration.application.host.clone(),
        port: configuration.application.port.clone(),
        oauth_server: configuration.application.oauth_server,
        client_id: Secret::new(configuration.application.client_id.clone()),
        oauth_redirect_host: configuration.application.oauth_redirect_host,
    });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(backend::routes::oauth::hello))
            .route("/oauth-redirect", web::get().to(backend::routes::oauth::oauth_login_redirect))
            .route("/logout", web::get().to(backend::routes::oauth::logout))
            .app_data(yoga_data.clone())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    // If the cookie is set as secure, it will only be transmitted when the connection is secure
                    // (using `https`).
                    .cookie_secure(false)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::hours(2)),
                    )
                    // the default is .cookie_http_only(true)
                    // the default is .cookie_content_security(CookieContentSecurity::Private) // encrypted but not signed
                    .build(),
            )
    })
    .bind((configuration.application.host, configuration.application.port.parse::<u16>().unwrap()))?
    .run()
    .await
}
