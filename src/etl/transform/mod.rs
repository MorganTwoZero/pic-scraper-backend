mod bcy;
mod lofter;
mod mihoyo_bbs;
mod pixiv;
mod twitter_home;
mod twitter_honkai;

pub use bcy::*;
pub use lofter::*;
pub use mihoyo_bbs::*;
pub use pixiv::*;
pub use twitter_home::*;
pub use twitter_honkai::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name  = "post_source", rename_all = "lowercase")]
pub enum PostSource {
    Twitter,
    Mihoyo,
    Pixiv,
    Bcy,
    Lofter,
}

#[derive(Serialize, Deserialize)]
pub struct Post {
    pub author: String,
    pub author_link: String,
    pub created: String,
    pub images_number: i32,
    pub post_link: String,
    pub preview_link: String,
    pub source: PostSource,
}