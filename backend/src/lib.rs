pub mod session_state;
pub mod routes;
pub mod configuration;

use std::collections::HashMap;

use oauth2::basic::BasicClient;
use secrecy::Secret;

pub struct YogaAppData {
    pub oauth_clients: HashMap<String, BasicClient>,
    pub client_id: Secret<String>,
    pub client_secret: Secret<String>,
    pub host: String,
    pub oauth_redirect_url: String,
    pub after_login_url: String,
    pub port: String,
}
