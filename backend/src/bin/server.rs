use actix_web::{
    cookie::{self, Key},
    get,
    http::header::ContentType,
    web, App, HttpResponse, HttpServer,
};

use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, SessionMiddleware,
};
use backend::session_state::TypedSession;
use dotenv;
use secrecy::{ExposeSecret, Secret};
use tracing_actix_web::TracingLogger;

struct YogaAppData {
    // this is the id from the application registered with FusionAuth
    client_id: Secret<String>,
    // this is the secret generated by FusionAuth
    client_secret: Secret<String>,
}

#[get("/")]
async fn hello(
    app_data: web::Data<YogaAppData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    // OAuth flow step 2.
    // The client redirects your browser to the authorization server.
    // It includes with the request, the client id, redirect uri, the response type, and one or
    // more scopes it needs.

    // the url of the OAuth server
    let auth_url = "aquiles.local";
    // client_id - this identifies the application you are loggin into
    // this is refered to as the client in OAuth vocabulary
    let client_id = app_data.client_id.expose_secret();
    // redirect_uri - this is the URL in your application to which the OAuth server will redirect
    // the user after they log in
    let redirect_ip = "matts-imac.local";
    let redirect_port = "3000";
    // state - optional, but it is useful for preventing various security isssues. Set to a large
    // random string or maybe some data with an appended random string. Then verify it after login.
    let state = session.generate_and_save_state().unwrap();
    // response_type - this indicates to OAuth server you are using the 'code' grant.
    let response_type = "code";
    // scope - optional, in some other modes it will be required, page 26 in book
    let scopes = urlencoding::encode("profile offline_access openi");
    // code_challenge - optional, provides support for PKCE
    let code_challenge = session.generate_and_save_code_challenge().unwrap();
    // code_challenge_method - optional, if your implement PKCE you must specify how your
    // code_challenge was created (plain or S256)
    // nonce - optional, used for OpenID Connect
    let nonce = session.generate_and_save_nonce().unwrap();

    let login_uri = format!("http://{}:9011/oauth2/authorize?", auth_url)
     + &format!("client_id={}&", client_id)
     + &format!("response_type={}&", response_type)
     + &format!("redirect_uri=") + &urlencoding::encode(&format!("http://{}:{}/", redirect_ip, redirect_port))
     + &format!("oauth-redirect&")
     + &format!("state={}&", state)
     + &format!("scope={}&", scopes)
     + &format!("code_challenge={}&", code_challenge)
     + &format!("code_challenge_method=S256&")
     + &format!("nonce={}", nonce);

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
            login_uri
        )))
}

    // a(href=fusionAuthURL+'/oauth2/logout/?client_id='+clientId) Logout

// OAuth flow step 5.
// The authorization server redirects back to the client using the redirect uri.
// Along with a temporary authorization code.

#[derive(serde::Deserialize)]
pub struct LoginRedirect {
    code: String,
    #[serde(alias = "userState")]
    user_state: String,
    state: String,
}

async fn oauth_login_redirect(
    login: web::Query<LoginRedirect>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    // redirect from the authorization server
    // code - authorization code the OAuth server created after the user logged in
    // it needs to be exchanged for tokens
    // state - this is the same value of the state parameter we passed to the OAuth server
    // this is echoed back to this application so that we can verify that the code
    // came from the correct location

    if let Ok(state) = session.get_state() {
        if let Some(state) = state {
            if login.state != state {
                let code_str = format!("got:      {}", login.state);
                let expc_str = format!("expected: {}", state);
                tracing::info!("State doesn't match.\n{}\n{}", code_str, expc_str);
            } else {
                tracing::info!("state matches yeah!");
            }
        }
    }

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
</body>
</html>"#,
            login.state, login.code
        )))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    dotenv::dotenv().ok();
    let client_id = dotenv::var("CLIENT_ID").unwrap().into();
    let client_secret = dotenv::var("CLIENT_SECRET").unwrap().into();

    let yoga_data = web::Data::new(YogaAppData {
        client_id,
        client_secret,
    });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(hello)
            .route("/oauth-redirect", web::get().to(oauth_login_redirect))
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

// http://127.0.0.1:8080/
