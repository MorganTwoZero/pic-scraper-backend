use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::transform::{Post, PostSource};

use super::DataSource;

#[derive(Serialize, Deserialize)]
struct User {
    nickname: String,
    uid: String,
    avatar_url: String,
}

#[derive(Serialize, Deserialize)]
struct Details {
    post_id: String,
    cover: Option<String>,
    created_at: i64,
    images: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct MihoyoPost {
    post: Details,
    user: User,
}

#[derive(Serialize, Deserialize)]
struct MihiyoData {
    list: Vec<MihoyoPost>,
}

#[derive(Serialize, Deserialize)]
pub struct MihoyoResponse {
    message: String,
    data: MihiyoData,
}

impl From<MihoyoPost> for Post {
    fn from(value: MihoyoPost) -> Self {
        let created = Utc
            .timestamp_opt(value.post.created_at, 0)
            .unwrap()
            .to_rfc3339();
        Self {
            post_link: format!("https://www.miyoushe.com/bh3/article/{}", value.post.post_id),
            preview_link: format!("{}?x-oss-process=image/resize,s_600/quality,q_80/auto-orient,0/interlace,1/format,jpg", value.post.cover.unwrap()),
            author_link: format!(
                "https://www.miyoushe.com/bh3/accountCenter/postList?id={}",
                value.user.uid
            ),
            author: value.user.nickname,
            created,
            source: PostSource::Mihoyo,
            images_number: value.post.images.len() as i32,
            tags: None,
            author_profile_image: Some(value.user.avatar_url),
        }
    }
}

impl From<MihoyoResponse> for Vec<Post> {
    fn from(value: MihoyoResponse) -> Self {
        value
            .data
            .list
            .into_iter()
            .filter(|p| p.post.cover.is_some())
            .map(Post::from)
            .collect()
    }
}

impl DataSource for MihoyoResponse {}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    const SAMPLE_JSON_PATH: &str = "../tests/assets/json/mihoyo.json";

    #[test]
    fn test_from_mihoyo_response_to_vec_posts() {
        let sample_json = fs::read_to_string(SAMPLE_JSON_PATH).expect("Unable to read the file");
        serde_json::from_str::<MihoyoResponse>(&sample_json).unwrap();
    }
}
