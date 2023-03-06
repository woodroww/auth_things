pub mod session_state;
pub mod routes;
pub mod configuration;

use std::collections::HashMap;
use oauth2::basic::BasicClient;

pub struct YogaAppData {
    pub oauth_clients: HashMap<String, BasicClient>,
    pub host: String,
    pub after_login_url: String,
    pub port: String,
}
