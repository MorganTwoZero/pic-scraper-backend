use chrono::{TimeZone, Utc};
use serde::Deserialize;

use crate::transform::{Post, PostSource};

use super::DataSource;

#[derive(Deserialize)]
pub struct PixivResponse {
    body: PixivBody,
}

#[derive(Deserialize)]
struct PixivBody {
    illusts: Vec<PixivIllust>,
}

#[derive(Deserialize)]
struct PixivIllust {
    url: String,
    tags: Vec<String>,
    upload_timestamp: i64,
    id: String,
    user_id: String,
    page_count: String,
    author_details: AuthorDetails,
}

#[derive(Deserialize)]
struct AuthorDetails {
    user_name: String,
}

impl From<PixivIllust> for Post {
    fn from(value: PixivIllust) -> Self {
        let created = Utc
            .timestamp_opt(value.upload_timestamp, 0)
            .unwrap()
            .to_rfc3339();
        Self {
            preview_link: value.url,
            post_link: format!("https://www.pixiv.net/en/artworks/{}", value.id),
            author_link: format!("https://www.pixiv.net/en/users/{}", value.user_id),
            author: value.author_details.user_name,
            created,
            source: PostSource::Pixiv,
            images_number: value.page_count.parse().unwrap(),
            tags: Some(value.tags),
            author_profile_image: None,
        }
    }
}

impl From<PixivResponse> for Vec<Post> {
    fn from(value: PixivResponse) -> Self {
        value.body.illusts.into_iter().map(Post::from).collect()
    }
}

impl DataSource for PixivResponse {}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    const PIXIV_JSON_PATH: &str = "../tests/assets/json/pixiv.json";

    #[test]
    fn test_from_pixiv_response_to_vec_posts() {
        let sample_json = fs::read_to_string(PIXIV_JSON_PATH).expect("Unable to read the file");
        serde_json::from_str::<PixivResponse>(&sample_json).unwrap();
    }
}
