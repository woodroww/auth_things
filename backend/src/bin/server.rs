use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use actix_web_lab::web::spa;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{self, Key},
    web, App, HttpServer,
};
use oauth2::{basic::BasicClient, RevocationUrl};
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

    //let fusion_uri = format!("http://{}/oauth2/authorize", configuration.application.oauth_server);
    //let token_endpoint = format!("http://{}/oauth2/token", configuration.application.oauth_server);

    let google_uri = String::from("https://accounts.google.com/o/oauth2/v2/auth");
    let token_uri = String::from("https://oauth2.googleapis.com/token");
    let revoke_uri = String::from("https://oauth2.googleapis.com/revoke");

    let redirect_uri = String::from("https://baeuerlin.net/oauth-redirect");

    /*
    let redirect_uri = format!(
        "https://{}:{}/oauth-redirect",
        configuration.application.oauth_redirect_host, configuration.application.port
    );
    */

    let client = BasicClient::new(
        ClientId::new(configuration.application.client_id.clone()),
        Some(ClientSecret::new(configuration.application.client_secret.clone())),
        AuthUrl::new(google_uri).unwrap(),
        Some(TokenUrl::new(token_uri).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap())
    .set_revocation_uri(RevocationUrl::new(revoke_uri).unwrap());

    let yoga_data = web::Data::new(YogaAppData {
        oauth_client: client,
        host: configuration.application.host.clone(),
        port: configuration.application.port.clone(),
        oauth_server: configuration.application.oauth_server,
        client_id: Secret::new(configuration.application.client_id.clone()),
        client_secret: Secret::new(configuration.application.client_secret.clone()),
        oauth_redirect_host: configuration.application.oauth_redirect_host,
    });

    let bind_address = (configuration.application.host, configuration.application.port.parse::<u16>().unwrap());

    tracing::info!("serving yogamat backend at https://{}:{}", bind_address.0, bind_address.1);

    /*
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("matt.test.key", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("matt.test.crt").unwrap();
    */

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/client-login", web::get().to(backend::routes::oauth::request_login_uri))
            .route("/oauth-redirect", web::get().to(backend::routes::oauth::oauth_login_redirect))
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
                    // If the cookie is set as secure, it will only be transmitted when the connection is secure
                    // (using `https`).
                    // .cookie_secure(false)
                    // the default is true
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::hours(2)),
                    )
                    // the default is .cookie_http_only(true)
                    // the default is .cookie_content_security(CookieContentSecurity::Private) // encrypted but not signed
                    .build(),
            )
    })
    //.bind_openssl(bind_address, builder)?
    .bind(bind_address)?
    .run()
    .await
}
