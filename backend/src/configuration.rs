// configuration of production and development setting with the help of the config crate
// https://docs.rs/config/0.13.3/config/
// Config organizes hierarchical or layered configurations for Rust applications.
// Config lets you set a set of default parameters and then extend them via merging in
// configuration from a variety of sources

use oauth2::basic::BasicClient;
use secrecy::Secret;

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: String,
    pub host: String,
    pub oauth_server: String,
    pub client_secret: String,
    pub client_id: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
}

pub struct YogaAppData {
    pub oauth_client: BasicClient,
    pub oauth_server: String,
    pub client_id: Secret<String>,
    pub host: String,
    pub port: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // dir in which the app is started
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    // (we must start where the config folder is located)
    let configuration_directory = base_path.join("configuration");
    // APP_ENVIRONMENT is set in Dockerfile or wherever
    // - to 'production' to make a production environment
    // - to 'local' to make a local developement environment
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    // create the config and deserialize into our Setting struct
    config::Config::builder()
        // get base settings from base.yaml
        .add_source(config::File::from(configuration_directory.join("base")).required(true))
        // add config settings from config file determined by environment
        .add_source(config::File::from(configuration_directory.join(environment.as_str())).required(true))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // eg. `APP_APPLICATION__PORT=5001` would set `Settings.application.port`
        // or `APP_DATABASE__USERNAME` would set `Settings.database.username`
        // This allows us to customize any value in our Settings struct using
        // environment variables, over-riding what is specified in our configuration files. 
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?
        .try_deserialize()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}