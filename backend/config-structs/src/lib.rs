use std::sync::Arc;

use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgPool};

#[derive(serde::Deserialize, Clone)]
pub struct SourcesUrls {
    pub pixiv: String,
    pub pixiv_details: String,
    pub pixiv_image: String,
    pub twitter_honkai: String,
    pub twitter_home: String,
    pub mihoyo: String,
    pub lofter: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct BlackList {
    pub authors: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct ScraperState {
    pub db_pool: PgPool,
    pub api_client: Client,
    pub blacklist: BlackList,
    pub sources_urls: SourcesUrls,
}

impl From<Arc<ScraperState>> for ScraperState {
    fn from(arc: Arc<ScraperState>) -> Self {
        (*arc).clone()
    }
}

#[derive(Clone)]
pub struct ApiState {
    pub db_pool: PgPool,
    pub api_client: Client,
    pub sources_urls: SourcesUrls,
}

impl From<Arc<ApiState>> for ApiState {
    fn from(arc: Arc<ApiState>) -> Self {
        (*arc).clone()
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApiClientHeaders {
    pub cookie: Secret<String>,
    pub authorization: Secret<String>,
    pub csrf_token: Secret<String>,
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
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }

    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
    }
}
