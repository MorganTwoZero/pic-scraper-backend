use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::etl::transform::{Post, PostSource};

#[derive(Serialize, Deserialize)]
struct AuthorDetails {
    user_id: String,
    user_name: String,
}

#[derive(Serialize, Deserialize)]
struct PixivIllust {
    url: String,
    tags: Vec<String>,
    upload_timestamp: i128,
    id: String,
    author_details: AuthorDetails,
    alt: String,
    title: String,
    page_count: String,
}

#[derive(Serialize, Deserialize)]
struct PixivBody {
    illusts: Vec<PixivIllust>,
}

#[derive(Serialize, Deserialize)]
pub struct PixivResponse {
    pub error: bool,
    pub message: String,
    body: PixivBody,
}

impl From<PixivIllust> for Post {
    fn from(value: PixivIllust) -> Self {
        let created = Utc
            .timestamp_opt(value.upload_timestamp.try_into().unwrap(), 0)
            .unwrap()
            .to_rfc3339()
            .split_once("+")
            .unwrap()
            .0
            .to_string();
        Self {
            preview_link: value.url,
            post_link: format!("https://www.pixiv.net/en/artworks/{}", value.id),
            author_link: format!(
                "https://www.pixiv.net/en/users/{}",
                value.author_details.user_id
            ),
            author: value.author_details.user_name,
            created,
            source: PostSource::Pixiv,
            images_number: value.page_count.parse().unwrap(),
        }
    }
}

impl From<PixivResponse> for Vec<Post> {
    fn from(value: PixivResponse) -> Self {
        value
            .body
            .illusts
            .into_iter()
            .map(|p| Post::from(p))
            .collect()
    }
}
