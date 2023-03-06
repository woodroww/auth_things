use crate::YogaAppData;
use crate::session_state::TypedSession;
use actix_web::{http::header::ContentType, web, HttpResponse, cookie::{
    time::{Duration, OffsetDateTime},
    Cookie, SameSite,
}, HttpRequest};
use oauth2::{basic::BasicTokenType, /*StandardRevocableToken*/};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope};
use oauth2::{EmptyExtraTokenFields, PkceCodeVerifier, StandardTokenResponse, TokenResponse};
use oauth2::basic::BasicClient;

#[actix_web::get("/client-login/{service}")]
pub async fn request_login_uri(
    app_data: web::Data<YogaAppData>,
    session: TypedSession,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    tracing::info!("request_login_uri");
    let oauth_provider = path.into_inner();

    tracing::info!("oauth_provider {}", oauth_provider);

    // OAuth flow
    // 2. The client (this app) redirects browser to the authorization server.
    // Through the Login link leading to auth_url.

    // save PKCE challenge in the session
    let (pkce_challenge, pkce_verifier): (PkceCodeChallenge, PkceCodeVerifier) =
        PkceCodeChallenge::new_random_sha256();
    session.set_pkce_verifier(pkce_verifier)?;

    // Generate the full authorization URL and a cross site request forgery token
    let (auth_url, csrf_token) = match oauth_provider.as_str() {
        "google" => {
            match app_data.oauth_clients.get("google") {
                Some(client) => {
                    session.insert_oauth_provider("google".to_string())?;
                    client
                        .authorize_url(CsrfToken::new_random)
                        //.add_scope(Scope::new("email".to_string()))
                        .add_scope(Scope::new("openid".to_string()))
                        .add_scope(Scope::new("profile".to_string()))
                        .set_pkce_challenge(pkce_challenge)
                        .url()
                }
                None => {
                    return Ok(HttpResponse::InternalServerError()
                        .body("oauth provider not in map"))
                }
            }
        }
        "fusion" => {
            match app_data.oauth_clients.get("fusion") {
                Some(client) => {
                    session.insert_oauth_provider("fusion".to_string())?;
                    client
                        .authorize_url(CsrfToken::new_random)
                        .add_scope(Scope::new("read".to_string()))
                        .add_scope(Scope::new("write".to_string()))
                        //.add_scope(Scope::new("offline_access".to_string())) // refresh tokens
                        .set_pkce_challenge(pkce_challenge)
                        .url()
                }
                None => {
                    return Ok(HttpResponse::InternalServerError()
                        .body("oauth provider not in map"))
                }
            }
        }
        "github" => {
            match app_data.oauth_clients.get("github") {
                Some(client) => {
                    session.insert_oauth_provider("github".to_string())?;
                    client
                        .authorize_url(CsrfToken::new_random)
                        .add_scope(Scope::new("read".to_string()))
                        .add_scope(Scope::new("write".to_string()))
                        //.add_scope(Scope::new("offline_access".to_string())) // refresh tokens
                        .set_pkce_challenge(pkce_challenge)
                        .url()
                }
                None => {
                    return Ok(HttpResponse::InternalServerError()
                        .body("oauth provider not in map"))
                }
            }
        }
        _ => {
            return Ok(HttpResponse::InternalServerError()
                .body("frontend requested invalid oauth provider"))
        }
    };

    // Save the state token to verify later.
    session.set_state(csrf_token)?;

    // send back the link to the auth provider
    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, Into::<String>::into(auth_url)))
        .body(""))
}

fn oauth_client<'a>(session: &TypedSession, app_data: &'a web::Data<YogaAppData>) -> Option<&'a BasicClient> {
    match session.get_oauth_provider() {
        Ok(session_ok) => match session_ok {
            Some(provider) => match app_data.oauth_clients.get(&provider) {
                Some(oauth_client) => {
                    Some(oauth_client)
                }
                None => {
                    tracing::error!("oauth provider not in map");
                    None
                }
            }
            None => {
                tracing::error!("recieved session returned None");
                None
            }
        }
        Err(session_error) => {
            tracing::error!("recieved session Err {}", session_error);
            None
        }
    }
}

#[actix_web::get("/logout")]
pub async fn logout(
    session: TypedSession,
    app_data: web::Data<YogaAppData>,
) -> Result<HttpResponse, actix_web::Error> {
    let token = session.get_access_token()?;

    // Since we are using session-based authentication a user is logged in if there is a valid
    // user id associated with the user_id key in the session state. To log out it is engough to
    // delete the session.
    // Removes session both client and server side.
    session.purge();

    // TODO: determine if we need to do this
    if let Some(token) = token {
        /*
        let revoke_token: StandardRevocableToken = match token.refresh_token() {
            Some(token) => token.into(),
            None => token.access_token().into(),
        };*/

        match oauth_client(&session, &app_data) {
            Some(oauth_client) => {
                oauth_client
                    .revoke_token(/*revoke_token*/token.into())
                    .unwrap()
                    .request(oauth2::reqwest::http_client)
                    .expect("Failed to revoke token");
            }
            None => {
                return Ok(HttpResponse::InternalServerError()
                    .body("session or oauth client error"))
            }
        }
    }

    Ok(HttpResponse::SeeOther()
        .insert_header((actix_web::http::header::LOCATION, "https://baeuerlin.net"))
        .finish())
}

pub async fn receive_token(
    app_data: web::Data<YogaAppData>,
    token: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    // oauth flow
    // 8. The client doesn't understand the token but can use it to send requests to the resource server

    // tracing::info!("matts fun token:\n{:#?}", token);

    // The access token issued by the authorization server.
    let jwt = token.access_token();
    session.set_access_token(jwt.clone())?;

    //let token_type = token.token_type();
    //let expires_in = token.expires_in();

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

    // back to frontend
    let cookie = Cookie::build("access_token", jwt.secret())
        .path("/")
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::now_utc().checked_add(Duration::minutes(60)))
        .finish();

    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, app_data.after_login_url.clone()))
        .content_type(ContentType::html())
        .cookie(cookie)
        .finish())
}

#[derive(serde::Deserialize)]
pub struct LoginRedirect {
    code: String,
    state: String,
}

#[actix_web::get("/oauth-redirect/local")]
pub async fn localhost_redirect(/*request: HttpRequest*/) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "http://127.0.0.1:3000/api/v1/oauth-redirect"))
        .content_type(ContentType::html())
        .finish())
}

#[actix_web::get("/oauth-redirect")]
pub async fn oauth_login_redirect(
    app_data: web::Data<YogaAppData>,
    login: web::Query<LoginRedirect>,
    //request: HttpRequest,
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

    let mut error_str = String::new();

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

        // OAuth flow
        // 6. The client then contacts the authorization server directly (not using the resource
        //    owners browser). Securely sends its client id, client secret, authorization code,

        let token_response = match oauth_client(&session, &app_data) {
            Some(oauth_client) => {
                Some(oauth_client
                    .exchange_code(AuthorizationCode::new(login.code.clone()))
                    .set_pkce_verifier(verifier)
                    .request_async(oauth2::reqwest::async_http_client)
                    .await)
            }
            None => {
                None
            }
        };

        // OAuth flow
        // 7. The authorization server verifies the data and respondes with an access token
        if let Some(token) = token_response {
            if let Ok(token) = token {
                // this is the happy path
                return receive_token(app_data, token, session).await;
            } else {
                error_str.push_str("<p>did not exchage code for token_response</p>")
            }
        } else {
            error_str.push_str("<p>none from token response</p>")
        }

    } else {

        if let Ok(s) = session.get_state() {
            if let Some(_state) = s {
            } else {
                error_str.push_str("<p>get_state Ok() but None, there is no session state</p>")
            }
        } else {
            error_str.push_str("<p>get_state Err(), there is no session state</p>")
        }
        if let Ok(v) = session.get_pkce_verifier() {
            if let Some(_verifier) = v {
            } else {
                error_str.push_str("<p>get_pkce_verifier Ok() but None, there is no pkce_verifier</p>")
            }
        } else {
            error_str.push_str("<p>get_pkce_verifier Err(), there is no pkce_verifier</p>")
        }
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
{}
</body>
</html>"#, error_str
        )))
}


