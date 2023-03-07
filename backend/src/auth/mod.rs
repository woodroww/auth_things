use oauth2::{ExtraTokenFields, Client, basic::{BasicErrorResponse, BasicTokenType, BasicTokenIntrospectionResponse, BasicRevocationErrorResponse, BasicClient}, StandardRevocableToken, StandardTokenResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoogleAuth {
    id_token: String,
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
