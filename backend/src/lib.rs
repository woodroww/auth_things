pub mod session_state;
pub mod routes;
pub mod configuration;
pub mod database;
pub mod auth;

use std::collections::HashMap;
use auth::{AuthClientType, AuthName};

pub struct YogaAppData {
    pub oauth_clients: HashMap<AuthName, AuthClientType>,
    pub host: String,
    pub after_login_url: String,
    pub port: String,
}
