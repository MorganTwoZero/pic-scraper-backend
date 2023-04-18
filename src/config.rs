use dotenvy::dotenv;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::ConnectOptions;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApiClientHeaders {
    pub cookie: Secret<String>,
    pub authorization: Secret<String>,
    pub csrf_token: Secret<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct BlackList {
    pub authors: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct SourcesUrls {
    pub pixiv: String,
    pub pixiv_details: String,
    pub pixiv_image: String,
    pub twitter_honkai: String,
    pub twitter_home: String,
    pub mihoyo: String,
    pub bcy: String,
    pub lofter: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app: ApplicationSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub blacklist: BlackList,
    pub sources_urls: SourcesUrls,
    pub headers: ApiClientHeaders,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }

    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    dotenv().ok();
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("config");

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

/// The possible runtime environment for our application.
pub enum Environment {
    Local,
    Prod,
    Stage,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Prod => "prod",
            Environment::Stage => "stage",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "prod" => Ok(Self::Prod),
            "stage" => Ok(Self::Stage),
            other => Err(format!(
                "{other} is not a supported environment. Use either `stage`, `local` or `prod`."
            )),
        }
    }
}
