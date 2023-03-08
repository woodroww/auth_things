use oauth2::{ExtraTokenFields, Client, basic::{BasicErrorResponse, BasicTokenType, BasicTokenIntrospectionResponse, BasicRevocationErrorResponse, BasicClient}, StandardRevocableToken, StandardTokenResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoogleAuth {
    pub id_token: String,
}

impl ExtraTokenFields for GoogleAuth {}

pub type GoogleClient = Client<
    BasicErrorResponse,
    GoogleTokenResponse,
    BasicTokenType,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
>;

pub type GoogleTokenResponse = StandardTokenResponse<GoogleAuth, BasicTokenType>;

pub enum AuthClientType {
    Google(GoogleClient),
    Basic(BasicClient),
}

#[derive(strum_macros::EnumString, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum AuthName {
    #[strum(serialize="google")]
    Google,
    #[strum(serialize="github")]
    GitHub,
    #[strum(serialize="fusion")]
    Fusion,
}

#[derive(Deserialize, Debug)]
pub struct GoogleClaims {
  pub aud: String,
  pub email: String,
  pub email_verified: bool,
  pub exp: usize,
  pub family_name: String,
  pub given_name: String,
  pub iat: usize,
  pub iss: String,
  pub locale: String,
  pub name: String,
  pub picture: String,
  pub sub: String,
}

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

