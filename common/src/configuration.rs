// configuration of production and development setting with the help of the config crate
// https://docs.rs/config/0.13.3/config/
// Config organizes hierarchical or layered configurations for Rust applications.
// Config lets you set a set of default parameters and then extend them via merging in
// configuration from a variety of sources

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: String,
    pub host: String,
    pub client_secret: String,
    pub client_id: String,
    pub oauth_redirect_url: String,
    pub after_login_url: String,
    pub oauth_url: String,
    pub token_url: String,
    pub revoke_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // dir in which the app is started
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    // (we must start where the config folder is located)
    let configuration_directory = base_path.join("../configuration");
    // APP_ENVIRONMENT is set in Dockerfile or local env
    // - 'ENV APP_ENVIRONMENT production' to make a production environment
    // - 'ENV APP_ENVIRONMENT local' to make a local developement environment
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "imac".into())
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
    //Local,
    IMac,
    Aquiles,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            //Environment::Local => "local",
            Environment::Production => "production",
            Environment::IMac => "imac",
            Environment::Aquiles => "aquiles",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            //"local" => Ok(Self::Local),
            "imac" => Ok(Self::IMac),
            "aquiles" => Ok(Self::Aquiles),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
