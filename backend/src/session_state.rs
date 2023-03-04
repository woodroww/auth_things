use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use oauth2::{CsrfToken, PkceCodeVerifier, AccessToken, RefreshToken};
use std::future::{ready, Ready};
use uuid::Uuid;

pub struct TypedSession(Session);

impl TypedSession {
    const STATE_KEY: &'static str = "oauth_state";
    const PKCE_VERIFIER_KEY: &'static str = "oauth_code_verifier";
    const TOKEN_KEY: &'static str = "access_token";
    const REFRESH_KEY: &'static str = "refresh_token";
    const USER_ID_KEY: &'static str = "user_id";
    const OAUTH_PROVIDER_KEY: &'static str = "oauth_provider";

    pub fn insert_oauth_provider(&self, provider: String) -> Result<(), SessionInsertError> {
        self.0.insert(Self::OAUTH_PROVIDER_KEY, provider)
    }
    pub fn get_oauth_provider(&self) -> Result<Option<String>, SessionGetError> {
        self.0.get(Self::OAUTH_PROVIDER_KEY)
    }

    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }
    pub fn get_user_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.0.get(Self::USER_ID_KEY)
    }

    // logout
    pub fn purge(&self) {
        self.0.purge()
    }
    // rotate
    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn set_state(&self, state: CsrfToken) -> Result<(), SessionInsertError> {
        self.0.insert(Self::STATE_KEY, state)
    }
    pub fn get_state(&self) -> Result<Option<CsrfToken>, SessionGetError> {
        self.0.get(Self::STATE_KEY)
    }

    pub fn set_pkce_verifier(&self, verifier: PkceCodeVerifier) -> Result<(), SessionInsertError> {
        self.0.insert(Self::PKCE_VERIFIER_KEY, verifier)
    }
    pub fn get_pkce_verifier(&self) -> Result<Option<PkceCodeVerifier>, SessionGetError> {
        self.0.get(Self::PKCE_VERIFIER_KEY)
    }

    pub fn set_access_token(&self, token: AccessToken) -> Result<(), SessionInsertError> {
        self.0.insert(Self::TOKEN_KEY, token)
    }
    pub fn get_access_token(
        &self,
    ) -> Result<Option<AccessToken>, SessionGetError>
    {
        self.0.get(Self::TOKEN_KEY)
    }

    pub fn set_refresh_token(&self, token: RefreshToken) -> Result<(), SessionInsertError> {
        self.0.insert(Self::REFRESH_KEY, token)
    }
    pub fn get_refresh_token(&self) -> Result<Option<RefreshToken>, SessionGetError> {
        self.0.get(Self::REFRESH_KEY)
    }
}

impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;

    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
