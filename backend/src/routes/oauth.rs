use crate::configuration::YogaAppData;
use crate::session_state::TypedSession;
use actix_web::{http::header::ContentType, web, HttpResponse, cookie::{
    time::{Duration, OffsetDateTime},
    Cookie, SameSite,
}};
use oauth2::basic::BasicTokenType;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope};
use oauth2::{EmptyExtraTokenFields, PkceCodeVerifier, StandardTokenResponse, TokenResponse};

use secrecy::ExposeSecret;
use serde_json::Value;

pub async fn request_login_uri(
    app_data: web::Data<YogaAppData>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    tracing::info!("request_login_uri");
    // OAuth flow
    // 2. The client (this app) redirects browser to the authorization server.
    // Through the Login link leading to auth_url.

    // save PKCE challenge in the session
    let (pkce_challenge, pkce_verifier): (PkceCodeChallenge, PkceCodeVerifier) =
        PkceCodeChallenge::new_random_sha256();
    session.set_pkce_verifier(pkce_verifier)?;

    // Generate the full authorization URL.
    // cross site request forgery token
    let (auth_url, csrf_token) = app_data
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        //.add_scope(Scope::new("read".to_string()))
        //.add_scope(Scope::new("write".to_string()))
        //.add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        //.add_scope(Scope::new("offline_access".to_string())) // refresh tokens
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Save the state token to verify later.
    session.set_state(csrf_token)?;

    // The web page the user sees with the link to the authorization server
    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, Into::<String>::into(auth_url)))
        .body(""))
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
}

pub async fn receive_token(
    _app_data: web::Data<YogaAppData>,
    token: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    // oauth flow
    // 8. The client doesn't understand the token but can use it to send requests to the resource server
    // tracing::info!("matts fun token:\n{:#?}", token);

    // The access token issued by the authorization server.
    let jwt = token.access_token();
    session.set_access_token(jwt.clone())?;

    let jambones = jwt.secret().clone();
    match serde_json::from_str(&jambones) {
        Ok(value) => {
            tracing::info!("json OK");
            match value {
                Value::Null => tracing::info!("Null"),
                Value::Bool(_) => tracing::info!("Bool"),
                Value::Number(_) => tracing::info!("Number"),
                Value::String(_) => tracing::info!("String"),
                Value::Array(_) => tracing::info!("Array"),
                Value::Object(_) => tracing::info!("Object"),
            }
        }
        Err(_) => tracing::info!("json un parsed"),
    }



    // JWT header
    //let header: Header = decode_header(&jwt).unwrap();
    //let kid = header.kid.clone().unwrap();
    // book has
    // accessToken, idToken, refreshToken
    // https://fusionauth.io/learn/expert-advice/tokens/jwt-components-explained
    // nonce can be verified if I knew how to send on, it is for openid

    match token.refresh_token() {
        Some(refresh) => {
            session.set_refresh_token(refresh.clone())?;
            tracing::info!("got a refresh token");
        }
        None => {
            tracing::info!("didn't got a refresh token");
        }
    }

    if let Some(scopes) = token.scopes() {
        for scope in scopes {
            tracing::info!("scope: {:?}", scope);
        }
    }

    /*
    let logout_uri = format!(
        "http://{}:{}/logout",
        app_data.oauth_redirect_host, app_data.port
    );
    */
    // yew path
    let after_login_url = format!("https://baeuerlin.net/login-success");
    let cookie = Cookie::build("email", "pretend_email")
        .path("/")
        .same_site(SameSite::Lax)
        .expires(OffsetDateTime::now_utc().checked_add(Duration::minutes(60)))
        .finish();

    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, after_login_url))
        .content_type(ContentType::html())
        .cookie(cookie)
        .finish())
}

pub async fn oauth_login_redirect(
    app_data: web::Data<YogaAppData>,
    login: web::Query<LoginRedirect>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    tracing::info!("oauth_login_redirect");
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
            // we may have been intercepted hacked or bamboozled
            // also need to send something back better than this
            let response = HttpResponse::SeeOther()
                .insert_header((actix_web::http::header::LOCATION, "/"))
                .finish();
            return Ok(response);
        }

        tracing::info!("States match, yeah!");
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
            // this is the happy path
            return receive_token(app_data, token, session).await;
        } else {
            tracing::info!("did not exchage code for token_response");
        }
    } else {
        tracing::info!("there is no session state or no verifier");
    }

    // this is going to be the error response
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Auth Error</title>
</head>
<body>
<p>Something went wrong with OAuth</p>
</body>
</html>"#,
        )))
}



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
/*
fn revoke_token() {
    // Revoke the obtained token
    let token_response = token_response.unwrap();
    let token_to_revoke: StandardRevocableToken = match token_response.refresh_token() {
        Some(token) => token.into(),
        None => token_response.access_token().into(),
    };

    client
        .revoke_token(token_to_revoke)
        .unwrap()
        .request(http_client)
        .expect("Failed to revoke token");
}
    */
