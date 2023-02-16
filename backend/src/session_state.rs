use actix_session::{Session, SessionExt, SessionInsertError, SessionGetError};
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use oauth2::{PkceCodeVerifier, CsrfToken};
use std::future::{Ready, ready};

pub struct TypedSession(Session);

impl TypedSession {
    const STATE_KEY: &'static str = "oauth_state";
    const PKCE_VERIFIER_KEY: &'static str = "oauth_code_verifier";

    pub fn purge(&self) {
        self.0.purge()
    }
    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn get_state(&self) -> Result<Option<CsrfToken>, SessionGetError> {
        self.0.get(Self::STATE_KEY)
    }
    pub fn set_state(&self, state: CsrfToken) -> Result<(), SessionInsertError> {
        self.0.insert(Self::STATE_KEY, state)
    }

    pub fn set_pkce_verifier(&self, verifier: PkceCodeVerifier) -> Result<(), SessionInsertError> {
        self.0.insert(Self::PKCE_VERIFIER_KEY, verifier)
    }
    pub fn get_pkce_verifier(&self) -> Result<Option<PkceCodeVerifier>, SessionGetError> {
        self.0.get(Self::PKCE_VERIFIER_KEY)
    }
}

impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;

    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
