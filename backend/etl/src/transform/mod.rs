mod lofter;
mod mihoyo_bbs;
mod pixiv;
mod twitter_home;
mod twitter_honkai;

pub use lofter::LofterResponse;
pub use mihoyo_bbs::MihoyoResponse;
pub use pixiv::PixivResponse;
use reqwest::Client;
pub use twitter_home::TwitterHomeResponse;
pub use twitter_honkai::TwitterHonkaiResponse;

use async_trait::async_trait;
use backon::ExponentialBuilder;
use backon::Retryable;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

use crate::Error;

#[derive(Serialize, Deserialize, sqlx::Type, Debug)]
#[sqlx(type_name = "post_source", rename_all = "lowercase")]
pub enum PostSource {
    Twitter,
    Mihoyo,
    Pixiv,
    Lofter,
    TwitterHome,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    pub author: String,
    pub author_link: String,
    pub author_profile_image: Option<String>,
    pub created: String,
    pub images_number: i32,
    pub post_link: String,
    pub preview_link: String,
    pub source: PostSource,
    pub tags: Option<Vec<String>>,
}

#[async_trait]
pub trait DataSource: Into<Vec<Post>> + DeserializeOwned {
    #[tracing::instrument(skip(client), level = "trace")]
    async fn request_and_parse(client: &Client, url: &str) -> Result<Vec<Post>, Error> {
        let response = (|| async { client.get(url).send().await })
            .retry(ExponentialBuilder::default())
            .notify(|err: &reqwest::Error, dur: Duration| {
                tracing::warn!("retrying {:?} after {:?}", err, dur);
            })
            .await?;
        let parsed = response
            .json::<Self>()
            .await
            .map_err(|e| {
                tracing::error!("Failed to parse a response. Error: {}. URL: {}", e, url);
                e
            })?
            .into();
        Ok(parsed)
    }
}

#[async_trait]
pub trait MultiUrlDataSource: DataSource {
    fn tags(url: &str) -> Vec<String>;

    async fn request_and_parse_multi(
        client: &Client,
        urls: Vec<String>,
    ) -> Result<Vec<Post>, Error>;
}
