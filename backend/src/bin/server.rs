use actix_cors::Cors;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{self, Key},
    http, web, App, HttpServer,
};
use backend::{configuration::{get_configuration, ApplicationSettings}, database::YogaDatabase, auth::{AuthClientType, AuthName}};
use backend::YogaAppData;
use oauth2::{basic::BasicClient, RevocationUrl, IntrospectionUrl};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::collections::HashMap;
use tracing_actix_web::TracingLogger;
use backend::auth::GoogleClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    tracing::info!("redirect_url: {}", configuration.application.oauth_redirect_url.clone());

    let database = YogaDatabase::new(configuration.database);
    let db = web::Data::new(database);

    let clients = setup_auth_providers(&configuration.application);

    let yoga_data = web::Data::new(YogaAppData {
        oauth_clients: clients,
        host: configuration.application.host.clone(),
        port: configuration.application.port.clone(),
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
            .app_data(db.clone())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
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

fn setup_auth_providers(application: &ApplicationSettings) -> HashMap<AuthName, AuthClientType> {
    let mut clients = HashMap::new();
    for provider in application.oauth_providers.iter() {
        let client_id_key = format!("{}_CLIENT_ID", provider.name.to_uppercase());
        let client_id = match std::env::var(client_id_key.clone()) {
            Ok(value) => Some(value),
            Err(_) => {
                tracing::error!("couldn't get {} from environment", client_id_key);
                None
            }
        };
        let client_secret_key = format!("{}_CLIENT_SECRET", provider.name.to_uppercase());
        let client_secret = match std::env::var(client_secret_key.clone()) {
            Ok(value) => Some(value),
            Err(_) => {
                tracing::error!("couldn't get {} from environment", client_secret_key);
                None
            }
        };
        if let (Some(id), Some(secret)) = (client_id, client_secret) {
            let auth_name: AuthName = match provider.name.as_str().try_into() {
                Ok(name) => name,
                Err(_) => {
                    tracing::error!("invalid auth provider name");
                    continue;
                }
            };
            match auth_name {
                AuthName::Google => {
                    let google_client = GoogleClient::new(
                        ClientId::new(id),
                        Some(ClientSecret::new(secret)),
                        AuthUrl::new(provider.oauth_url.clone()).unwrap(),
                        Some(TokenUrl::new(provider.token_url.clone()).unwrap()),
                    ).set_redirect_uri(
                            RedirectUrl::new(application.oauth_redirect_url.clone()).unwrap(),
                        )
                        .set_revocation_uri(RevocationUrl::new(provider.revoke_url.clone()).unwrap())
                        .set_introspection_uri(IntrospectionUrl::new(provider.introspection_url.clone()).unwrap());
                    clients.insert(auth_name, AuthClientType::Google(google_client));
                },
                AuthName::GitHub | AuthName::Fusion => {
                    let client = BasicClient::new(
                        ClientId::new(id),
                        Some(ClientSecret::new(secret)),
                        AuthUrl::new(provider.oauth_url.clone()).unwrap(),
                        Some(TokenUrl::new(provider.token_url.clone()).unwrap()),
                    )
                        .set_redirect_uri(
                            RedirectUrl::new(application.oauth_redirect_url.clone()).unwrap(),
                        )
                        .set_revocation_uri(RevocationUrl::new(provider.revoke_url.clone()).unwrap())
                        .set_introspection_uri(IntrospectionUrl::new(provider.introspection_url.clone()).unwrap());
                    clients.insert(auth_name, AuthClientType::Basic(client));
                }
            }
        }
    }
    clients
}
