use std::collections::HashMap;

use crate::auth::{AuthClientType, GoogleAuth, GoogleClaims, GoogleClient};
use crate::session_state::TypedSession;
use crate::{auth::AuthName, YogaAppData};
use actix_web::{
    cookie::{
        time::{Duration, OffsetDateTime},
        Cookie, SameSite,
    },
    http::header::ContentType,
    web, HttpResponse,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, TokenData};
use oauth2::{
    basic::{BasicClient, BasicTokenType},
    AccessToken, TokenIntrospectionResponse,
};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, Scope};
use oauth2::{EmptyExtraTokenFields, PkceCodeVerifier, StandardTokenResponse, TokenResponse};

#[actix_web::get("/client-login/{service}")]
pub async fn request_login_uri(
    app_data: web::Data<YogaAppData>,
    session: TypedSession,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let oauth_provider: AuthName = match path.into_inner().as_str().try_into() {
        Ok(name) => name,
        Err(_) => {
            tracing::error!("couldn't convert str into AuthName");
            return Ok(HttpResponse::InternalServerError().body("oauth provider not in map"));
        }
    };

    // OAuth flow
    // 2. The client (this app) redirects browser to the authorization server.
    // Through the Login link leading to auth_url.

    // save PKCE challenge in the session
    let (pkce_challenge, pkce_verifier): (PkceCodeChallenge, PkceCodeVerifier) =
        PkceCodeChallenge::new_random_sha256();
    session.set_pkce_verifier(pkce_verifier)?;

    // Generate the full authorization URL and a cross site request forgery token
    let (auth_url, csrf_token) = match oauth_provider {
        AuthName::Google => match app_data.oauth_clients.get(&oauth_provider) {
            Some(client) => {
                session.insert_oauth_provider(AuthName::Google)?;
                if let AuthClientType::Google(google_client) = client {
                    google_client
                        .authorize_url(CsrfToken::new_random)
                        .add_scope(Scope::new("openid".to_string()))
                        .add_scope(Scope::new("profile".to_string()))
                        .add_scope(Scope::new("email".to_string()))
                        .add_extra_param("access_type", "offline")
                        .set_pkce_challenge(pkce_challenge)
                        .url()
                } else {
                    panic!();
                }
            }
            None => {
                return Ok(HttpResponse::InternalServerError().body("oauth provider not in map"))
            }
        },
        AuthName::Fusion => {
            match app_data.oauth_clients.get(&oauth_provider) {
                Some(client) => {
                    session.insert_oauth_provider(AuthName::Fusion)?;
                    if let AuthClientType::Basic(fusion_client) = client {
                        fusion_client
                            .authorize_url(CsrfToken::new_random)
                            .add_scope(Scope::new("read".to_string()))
                            .add_scope(Scope::new("write".to_string()))
                            .add_scope(Scope::new("offline".to_string())) // refresh tokens
                            .set_pkce_challenge(pkce_challenge)
                            .url()
                    } else {
                        panic!();
                    }
                }
                None => {
                    return Ok(HttpResponse::InternalServerError().body("oauth provider not in map"))
                }
            }
        }
        AuthName::GitHub => match app_data.oauth_clients.get(&oauth_provider) {
            Some(client) => {
                session.insert_oauth_provider(AuthName::GitHub)?;
                if let AuthClientType::Basic(github_client) = client {
                    github_client
                        .authorize_url(CsrfToken::new_random)
                        .add_scope(Scope::new("read".to_string()))
                        .add_scope(Scope::new("write".to_string()))
                        .set_pkce_challenge(pkce_challenge)
                        .url()
                } else {
                    panic!();
                }
            }
            None => {
                return Ok(HttpResponse::InternalServerError().body("oauth provider not in map"))
            }
        },
    };

    // Save the state token to verify later.
    session.set_state(csrf_token)?;

    //tracing::info!("login: {}", Into::<String>::into(auth_url.clone()));
    // send back the link to the auth provider
    Ok(HttpResponse::Found()
        .append_header((
            actix_web::http::header::LOCATION,
            Into::<String>::into(auth_url),
        ))
        .body(""))
}

fn oauth_client<'a>(
    session: &TypedSession,
    app_data: &'a web::Data<YogaAppData>,
) -> Option<&'a AuthClientType> {
    match session.get_oauth_provider() {
        Ok(session_ok) => match session_ok {
            Some(provider_name) => match app_data.oauth_clients.get(&provider_name) {
                Some(oauth_client) => Some(oauth_client),
                None => {
                    tracing::error!("oauth provider not in map");
                    None
                }
            },
            None => {
                tracing::error!("recieved session returned None");
                None
            }
        },
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

    /*
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
    */
    Ok(HttpResponse::SeeOther()
        .insert_header((actix_web::http::header::LOCATION, "https://baeuerlin.net"))
        .finish())
}

async fn introspect(
    access_token: &AccessToken,
    session: TypedSession,
    app_data: web::Data<YogaAppData>,
) -> Result<(), actix_web::Error> {
    match oauth_client(&session, &app_data) {
        Some(client) => match client {
            AuthClientType::Google(google) => match google.introspect(access_token) {
                Ok(request) => {
                    match request
                        .request_async(oauth2::reqwest::async_http_client)
                        .await
                    {
                        Ok(response) => {
                            let what = response.scopes();
                        }
                        Err(error) => {
                            tracing::error!("introspection request failed {}", error);
                        }
                    }
                }
                Err(error) => {
                    tracing::error!("introspection creation failed {}", error);
                }
            },
            AuthClientType::Basic(basic) => match basic.introspect(access_token) {
                Ok(request) => {
                    match request
                        .request_async(oauth2::reqwest::async_http_client)
                        .await
                    {
                        Ok(response) => {
                            let what = response.scopes();
                        }
                        Err(error) => {
                            tracing::error!("introspection request failed {}", error);
                        }
                    }
                }
                Err(error) => {
                    tracing::error!("introspection creation failed {}", error);
                }
            },
        },
        None => {}
    }
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct LoginRedirect {
    code: String,
    state: String,
}

#[actix_web::get("/oauth-redirect")]
pub async fn oauth_login_redirect(
    app_data: web::Data<YogaAppData>,
    login: web::Query<LoginRedirect>,
    //request: HttpRequest,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    //tracing::info!("oauth_login_redirect");
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

        // OAuth flow
        // 6. The client then contacts the authorization server directly (not using the resource
        //    owners browser). Securely sends its client id, client secret, authorization code,
        match oauth_client(&session, &app_data) {
            Some(oauth_client) => match oauth_client {
                AuthClientType::Basic(basic) => {
                    return basic_exchange(
                        app_data.clone(),
                        session,
                        login.code.clone(),
                        verifier,
                        basic,
                    )
                    .await;
                }
                AuthClientType::Google(google) => {
                    return google_exchange(
                        app_data.clone(),
                        session,
                        login.code.clone(),
                        verifier,
                        google,
                    )
                    .await;
                }
            },
            None => {
                panic!();
            }
        }
    }
    let error_str = session_error(session);
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
</html>"#,
            error_str
        )))
}

fn session_error(session: TypedSession) -> String {
    let mut error_str = String::new();
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
    error_str
}

async fn basic_exchange(
    app_data: web::Data<YogaAppData>,
    session: TypedSession,
    code: String,
    verifier: PkceCodeVerifier,
    basic: &BasicClient,
) -> Result<HttpResponse, actix_web::Error> {
    let token_response = basic
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    if let Ok(token) = token_response {
        // this is the happy path
        return receive_token(app_data, token, session).await;
    } else {
        // TODO error_str.push_str("<p>did not exchage code for token_response</p>")
        panic!()
    }
}

async fn receive_token(
    app_data: web::Data<YogaAppData>,
    token: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    // oauth flow
    // 8. The client doesn't understand the token but can use it to send requests to the resource server

    //tracing::info!("matts fun token:\n{:#?}", token);

    // The access token issued by the authorization server.
    let jwt = token.access_token();
    session.set_access_token(jwt.clone())?;

    //let extra = token.extra_fields();
    //let token_type = token.token_type();
    //let expires_in = token.expires_in();

    if let Some(refresh) = token.refresh_token() {
        session.set_refresh_token(refresh.clone())?;
    }

    let after_login_url = app_data.after_login_url.clone();
    //let what = introspect(jwt, session, app_data).await?;

    // back to frontend
    let cookie = Cookie::build("access_token", jwt.secret())
        .path("/")
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::now_utc().checked_add(Duration::minutes(60)))
        .finish();

    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, after_login_url))
        .content_type(ContentType::html())
        .cookie(cookie)
        .finish())
}

async fn google_exchange(
    app_data: web::Data<YogaAppData>,
    session: TypedSession,
    code: String,
    verifier: PkceCodeVerifier,
    google: &GoogleClient,
) -> Result<HttpResponse, actix_web::Error> {
    let token_response = google
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    // OAuth flow
    // 7. The authorization server verifies the data and respondes with an access token
    if let Ok(token) = token_response {
        // this is the happy path
        return receive_google_token(app_data, token, session).await;
    } else {
        // TODO error_str.push_str("<p>did not exchage code for token_response</p>")
        panic!()
    }
}

async fn receive_google_token(
    app_data: web::Data<YogaAppData>,
    token: StandardTokenResponse<GoogleAuth, BasicTokenType>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    // oauth flow
    // 8. The client doesn't understand the token but can use it to send requests to the resource server

    //tracing::info!("matts fun token:\n{:#?}", token);

    // The access token issued by the authorization server.
    let jwt = token.access_token();
    session.set_access_token(jwt.clone())?;

    let extra: &GoogleAuth = token.extra_fields();
    match verify_google_id_token(&extra.id_token).await {
        Ok(claims) => {
            tracing::info!("verify reqwest ok {:#?}", claims);
        }
        Err(error) => {
            tracing::error!("verify reqwest error {}", error);
        }
    }

    //let token_type = token.token_type();
    //let expires_in = token.expires_in();

    match token.refresh_token() {
        Some(refresh) => {
            session.set_refresh_token(refresh.clone())?;
        }
        None => {}
    }

    let after_login_url = app_data.after_login_url.clone();
    //let what = introspect(jwt, session, app_data).await?;
    
    // does this belong here? it belongs somewhere
    session.renew();

    // back to frontend
    let cookie = Cookie::build("access_token", jwt.secret())
        .path("/")
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::now_utc().checked_add(Duration::minutes(60)))
        .finish();

    Ok(HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, after_login_url))
        .content_type(ContentType::html())
        .cookie(cookie)
        .finish())
}

// An ID Token is a JWT (JSON Web Token), that is, a cryptographically signed Base64-encoded JSON object.
// You need to validate all ID tokens on your server unless you know that they came directly from Google
// The Discovery document for Google's OpenID Connect service may be retrieved from:
// https://accounts.google.com/.well-known/openid-configuration
// get the urls from this document, it has the auth and token endpoints too
// Google-issued tokens are signed using one of the certificates found at the URI specified in
// the jwks_uri metadata value of the Discovery document.
// "jwks_uri": "https://www.googleapis.com/oauth2/v3/certs",
// jwks_uri lists keys each with:
// alg kty n e use kid

// kid = the ID of the key used to sign this token
// the id_token header should have a kid indicating the correct key in the jwks

#[derive(thiserror::Error, Debug)]
pub enum VerifyTokenError {
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("id_token has no kid")]
    NoKid,
    #[error("id-token header kid not found in jwks")]
    KidNotFound,
    #[error("jsonwebtoken error")]
    JsonwebTokenError(#[from] jsonwebtoken::errors::Error),
}

async fn verify_google_id_token(id_token: &str) -> Result<GoogleClaims, VerifyTokenError> {
    let jwks = reqwest::get("https://www.googleapis.com/oauth2/v3/certs")
        .await?
        .json::<HashMap<String, Vec<HashMap<String, String>>>>()
        .await?;
    let header = jsonwebtoken::decode_header(&id_token).unwrap();
    if let Some(token_kid) = header.kid {
        let jwks_keys = jwks.get("keys").unwrap();
        for key in jwks_keys {
            match key.get("kid") {
                Some(kid) => {
                    if kid == &token_kid {
                        let modulus = key.get("n").unwrap();
                        let exponent = key.get("e").unwrap();
                        match jsonwebtoken::decode::<GoogleClaims>(
                            &id_token,
                            &DecodingKey::from_rsa_components(modulus, exponent).expect("this to work"),
                            &Validation::new(Algorithm::RS256),
                        ) {
                            Ok(token) => {
                                let token: TokenData<GoogleClaims> = token;
                                return Ok(token.claims);
                            }
                            Err(err) => {
                                tracing::error!("jsonwebtoken error {}", err);
                                return Err(VerifyTokenError::JsonwebTokenError(err));
                            }
                        };
                    }
                }
                None => {
                }
            }
        }
        return Err(VerifyTokenError::KidNotFound);
    } else {
        tracing::error!("id_token has no kid in header");
        return Err(VerifyTokenError::NoKid);
    }
}


    /*
    use base64::{Engine as _, alphabet, engine::{self, general_purpose}};
    let bytes = engine::GeneralPurpose::new(
        &alphabet::URL_SAFE,
        general_purpose::NO_PAD)
        .decode(id_token).unwrap();
    println!("BYTES!!!\n{:?}", bytes);
    */

