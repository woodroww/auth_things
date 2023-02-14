use actix_session::{Session, SessionExt, SessionInsertError, SessionGetError};
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::future::{Ready, ready};
use rand::Rng;
use sha2::{Digest, Sha256};
use urlencoding::encode;
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};

const CUSTOM_ENGINE: base64::engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);


pub struct TypedSession(Session);


impl TypedSession {
    const STATE_KEY: &'static str = "oauth_state";
    const CODE_CHALLENGE_KEY: &'static str = "oauth_code_verifier";
    const NONCE_KEY: &'static str = "nonce";

    pub fn renew(&self) {
        self.0.renew();
    }
    pub fn get_state(&self) -> Result<Option<String>, SessionGetError> {
        self.0.get(Self::STATE_KEY)
    }
    pub fn get_code_challenge(&self) -> Result<Option<String>, SessionGetError> {
        self.0.get(Self::CODE_CHALLENGE_KEY) 
    }
    pub fn get_nonce(&self) -> Result<Option<String>, SessionGetError> {
        self.0.get(Self::NONCE_KEY) 
    }
    pub fn generate_and_save_state(&self) -> Result<String, SessionInsertError> {
        let result = TypedSession::make_random_state();
        self.0.insert(Self::STATE_KEY, result.clone())?;
        Ok(result)
    }
    pub fn generate_and_save_code_challenge(&self) -> Result<String, SessionInsertError> {
        let result = TypedSession::make_random_state();
        self.0.insert(Self::CODE_CHALLENGE_KEY, result.clone())?; 
        Ok(result)
    }
    pub fn generate_and_save_nonce(&self) -> Result<String, SessionInsertError> {
        let result = TypedSession::make_random_state();
        self.0.insert(Self::NONCE_KEY, result.clone())?; 
        Ok(result)
    }
    pub fn log_out(self) {
        self.0.purge()
    }

    fn make_random_state() -> String {
        let mut generator = rand::thread_rng();
        let rando: u64 = generator.gen();
        let rando_string = rando.to_string();
        let oauth_code = encode(&rando_string);
        let mut hasher = Sha256::new();
        hasher.update(oauth_code.as_bytes());
        let hashed_oauth_code = hasher.finalize();
        let base64_encoded = CUSTOM_ENGINE.encode(hashed_oauth_code);
        //let url_encoded_code_challenge = encode(&base64_encoded);
        base64_encoded
    }
}

impl FromRequest for TypedSession {
    type Error = <Session as FromRequest>::Error;

    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}
