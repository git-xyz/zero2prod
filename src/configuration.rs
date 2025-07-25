use sqlx::ConnectOptions;
use  secrecy::{ExposeSecret, Secret};
// use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use crate::domain::{SubscriberEmail};


#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    // pub application_port: u16,
    pub application: ApplicationSetting,
    pub email_client: EmailClientSettings
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSetting{
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String
}

pub enum Environment {
    Local,
    Production
}


pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir()
        .expect("Failed to determine the current directory");
    let configurtion_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let enviroment_filename = format!("{}.yaml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(
            config::File::from(configurtion_directory.join("base.yaml"))
        )
        .add_source(
            config::File::from(configurtion_directory.join(enviroment_filename))
        )
        .add_source(
            config::Environment::with_prefix("APP")  
            .prefix_separator("_")
            .separator("__")
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {

    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
        //format!("postgres://{}:{}@{}:{}/{}", self.username, self.password, self.host, self.port, self.database_name)
    }
    
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
        //format!("postgres://{}:{}@{}:{}", self.username, self.password, self.host, self.port)
    }
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer  
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
    pub fn with_db(&self) -> PgConnectOptions {
        let options = self.without_db().database(&self.database_name);
        options.clone().log_statements(tracing_log::log::LevelFilter::Trace);
        options
    }
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production"
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`",
                other
            ))
        }
    }
    
}