mod bcy;
mod lofter;
mod mihoyo_bbs;
mod pixiv;
mod twitter_home;
mod twitter_honkai;

pub use bcy::BcyResponse;
pub use lofter::LofterResponse;
pub use mihoyo_bbs::MihoyoResponse;
pub use pixiv::PixivResponse;
use reqwest::Client;
pub use twitter_home::TwitterHomeResponse;
pub use twitter_honkai::TwitterHonkaiResponse;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use async_trait::async_trait;

use crate::Error;

#[derive(Serialize, Deserialize, sqlx::Type, Debug)]
#[sqlx(type_name  = "post_source", rename_all = "lowercase")]
pub enum PostSource {
    Twitter,
    Mihoyo,
    Pixiv,
    Bcy,
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
    pub tags: Option<Vec<String>>
}

#[async_trait]
pub trait DataSource: Into<Vec<Post>> + DeserializeOwned {
    fn url() -> &'static str;

    async fn request_and_parse(client: &Client) -> Result<Vec<Post>, Error> {
        let response = client.get(Self::url()).send().await?;
        let parsed = response.json::<Self>().await?.into();
        Ok(parsed)
    }
}

#[async_trait]
pub trait MultiUrlDataSource: DataSource {
    fn urls() -> Vec<String>;

    async fn request_and_parse_multi(client: &Client) -> Result<Vec<Post>, Error>;
}