use actix_web::{
    cookie::{self, Key},
    get,
    http::header::ContentType,
    web, App, HttpResponse, HttpServer,
};

use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use backend::session_state::TypedSession;
use dotenv;
use tracing_actix_web::TracingLogger;

use oauth2::{basic::BasicClient, PkceCodeVerifier};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl,
    Scope, TokenUrl,
};

struct YogaAppData {
    oauth_client: BasicClient,
    host: String,
    port: String,
}

#[get("/")]
async fn hello(
    app_data: web::Data<YogaAppData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    // OAuth flow
    // 2. The client (this app) redirects browser to the authorization server.
    // Through the Login link leading to auth_url.

    // save PKCE challenge in the session
    let (pkce_challenge, pkce_verifier): (PkceCodeChallenge, PkceCodeVerifier) = PkceCodeChallenge::new_random_sha256();
    session.set_pkce_verifier(pkce_verifier)?;

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = app_data
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Save the state token to verify later.
    session.set_state(csrf_token);

    // The web page the user sees with the link to the authorization server
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
</head>
<body>
Hello world! <a href={}>Login</a>
</body>
</html>"#,
            auth_url
        )))
}

#[derive(serde::Deserialize)]
pub struct LoginRedirect {
    code: String,
    state: String,
}

/*
async fn logout(
    session: TypedSession,
    app_data: web::Data<YogaAppData>,
) -> Result<HttpResponse, actix_web::Error> {
    session.purge();
    let logout_endpoint = format!("http://{}/oauth2/logout?client_id={}", app_data.oauth_server, app_data.client_id.expose_secret());

    let response = reqwest::Client::new()
        .get(&logout_endpoint)
        .send()
        .await.unwrap();

    let status = response.status();
    let response_text = response.text().await.unwrap();
    if status.is_success() {
        println!("success token response body {}", response_text);
    } else {
        println!("status: {}\n{}", status, response_text);
    }

    // a(href=fusionAuthURL+'/oauth2/logout/?client_id='+clientId) Logout

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Logged Out</title>
</head>
<body>
<p>You have been logged out.</p>
<p>Bye.</p>
</body>
</html>"#
        )))
}
        */

async fn oauth_login_redirect(
    app_data: web::Data<YogaAppData>,
    login: web::Query<LoginRedirect>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {

    // OAuth flow
    // 5. The authorization server redirects back to the client using the redirect uri. Along with
    //    a temporary authorization code.

    // code - authorization code the OAuth server created after the user logged in
    // it needs to be exchanged for tokens
    // state - this is the same value of the state parameter we passed to the OAuth server
    // this is echoed back to this application so that we can verify that the code
    // came from the correct location

    if let (Ok(Some(state)), Ok(Some(verifier))) = (session.get_state(), session.get_pkce_verifier()) {
        // verify the states are the same
        if login.state != *state.secret() {
            tracing::info!("State doesn't match. Something is terribly wrong.");
            // we have been intercepted hacked bamboozled,
            // also need to send something back better than this
            let response = HttpResponse::SeeOther()
                .insert_header((actix_web::http::header::LOCATION, "/"))
                .finish();
            return Ok(response);
        }

        // OAuth flow
        // 6. The client then contacts the authorization server directly (not using the resource
        //    owners browser). Securely sends its client id, client secret, authorization code,
        let token_response = app_data
            .oauth_client
            .exchange_code(AuthorizationCode::new(login.code.clone()))
            .set_pkce_verifier(verifier)
            .request_async(oauth2::reqwest::async_http_client).await;

        // OAuth flow
        // 7. The authorization server verifies the data and respondes with an access token
        if let Ok(token) = token_response {
            println!("matts fun token:\n{:#?}", token);
        }

    } else {
        tracing::info!("there is no session state or no verifier");
        panic!();
    }

    let logout_uri = format!("http://{}:{}/logout", app_data.host, app_data.port);

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Logged In</title>
</head>
<body>
<p>You have been logged in.</p>
<p>user_state {}</p>
<p>code {}</p>
<a href={}>Logout</a>
</body>
</html>"#,
            login.state, login.code, logout_uri,
        )))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    dotenv::dotenv().ok();
    let client_id = dotenv::var("CLIENT_ID").unwrap();
    let client_secret = dotenv::var("CLIENT_SECRET").unwrap();
    let oauth_server = "aquiles.local:9011".to_string();
    let host = "matts-imac.local".to_string();
    let port = "3000".to_string();

    let fusion_uri = format!("http://{}/oauth2/authorize", oauth_server);
    let token_endpoint = format!("http://{}/oauth2/token", oauth_server);
    let redirect_uri = format!("http://{}:{}/oauth-redirect", host, port);

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(fusion_uri).unwrap(),
        Some(TokenUrl::new(token_endpoint).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap());

    let yoga_data = web::Data::new(YogaAppData {
        oauth_client: client,
        host,
        port
    });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(hello)
            .route("/oauth-redirect", web::get().to(oauth_login_redirect))
            //.route("/logout", web::get().to(logout))
            //.route("/token-callback", web::get().to(token_exchange))
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
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

