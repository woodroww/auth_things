use crate::configuration::YogaAppData;
use crate::session_state::TypedSession;
use actix_web::{
    http::header::ContentType, web, HttpResponse,
};
use oauth2::PkceCodeVerifier;
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope,
};

use secrecy::ExposeSecret;

pub async fn hello(
    app_data: web::Data<YogaAppData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    // OAuth flow
    // 2. The client (this app) redirects browser to the authorization server.
    // Through the Login link leading to auth_url.

    // save PKCE challenge in the session
    let (pkce_challenge, pkce_verifier): (PkceCodeChallenge, PkceCodeVerifier) =
        PkceCodeChallenge::new_random_sha256();
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
    session.set_state(csrf_token)?;

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

pub async fn logout(
    session: TypedSession,
    app_data: web::Data<YogaAppData>,
) -> Result<HttpResponse, actix_web::Error> {
    session.purge();
    let logout_endpoint = format!(
        "http://{}/oauth2/logout?client_id={}",
        app_data.oauth_server,
        app_data.client_id.expose_secret()
    );

    Ok(HttpResponse::SeeOther()
        .insert_header((actix_web::http::header::LOCATION, logout_endpoint))
        .finish())
    /*
        reqwest::Client::new()
            .get(&logout_endpoint)
            .send()
            .await.unwrap()

        let status = response.status();
        let response_text = response.text().await.unwrap();
        if status.is_success() {
            println!("success token response body {}", response_text);
        } else {
            println!("status: {}\n{}", status, response_text);
        }

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
            */
}

pub async fn oauth_login_redirect(
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

    if let (Ok(Some(state)), Ok(Some(verifier))) =
        (session.get_state(), session.get_pkce_verifier())
    {
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
            .request_async(oauth2::reqwest::async_http_client)
            .await;

        // OAuth flow
        // 7. The authorization server verifies the data and respondes with an access token
        if let Ok(token) = token_response {
            println!("matts fun token:\n{:#?}", token);
        }
    } else {
        tracing::info!("there is no session state or no verifier");
        panic!();
    }

    let logout_uri = format!("http://{}:{}/logout", app_data.oauth_redirect_host, app_data.port);

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
