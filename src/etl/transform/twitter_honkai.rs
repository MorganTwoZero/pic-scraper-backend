use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::{
    etl::transform::{Post, PostSource},
    Error,
};

use super::DataSource;

#[derive(Deserialize, Serialize)]
struct TweetHonkai {
    created_at: String,
    entities: Entities,
    user: User,
}

#[derive(Deserialize, Serialize)]
struct Entities {
    hashtags: Option<Vec<Hashtag>>,
    media: Option<Vec<Media>>,
}

#[derive(Deserialize, Serialize)]
struct Hashtag {
    text: String,
}

#[derive(Deserialize, Serialize)]
struct Media {
    media_url_https: String,
    expanded_url: String,
}

#[derive(Deserialize, Serialize)]
struct User {
    name: String,
    screen_name: String,
    profile_image_url_https: String,
}

#[derive(Deserialize, Serialize)]
pub struct TwitterHonkaiResponse {
    statuses: Vec<TweetHonkai>,
}

impl TryFrom<TweetHonkai> for Post {
    type Error = Error;

    fn try_from(value: TweetHonkai) -> Result<Self, Self::Error> {
        let created = DateTime::parse_from_str(&value.created_at, "%a %b %d %H:%M:%S %z %Y")
            .ok()
            .ok_or(Error::Parsing)?
            .to_rfc3339();
        let media = value
            .entities
            .media
            .expect("Checked in From<TwitterHomeResponse>");
        let main_pic = <Vec<Media> as AsRef<[Media]>>::as_ref(media.as_ref())
            .get(0)
            .ok_or(Error::Parsing)?;
        let tags = value.entities.hashtags.map(|tags| {
            tags.into_iter()
                .map(|tag| tag.text)
                .collect::<Vec<String>>()
        });
        Ok(Self {
            preview_link: main_pic.media_url_https.to_string(),
            post_link: main_pic.expanded_url.replace("/photo/1", ""),
            author_link: format!("https://twitter.com/{}", value.user.screen_name),
            author: format!("{}@{}", value.user.name, value.user.screen_name),
            created,
            source: PostSource::Twitter,
            images_number: media.len() as i32,
            tags,
            author_profile_image: Some(value.user.profile_image_url_https),
        })
    }
}

impl From<TwitterHonkaiResponse> for Vec<Post> {
    fn from(value: TwitterHonkaiResponse) -> Self {
        value
            .statuses
            .into_iter()
            .filter(|tw| tw.entities.media.is_some())
            .filter_map(|tw| Post::try_from(tw).ok())
            .collect()
    }
}

impl DataSource for TwitterHonkaiResponse {
    fn url() -> &'static str {
        "https://api.twitter.com/1.1/search/tweets.json?result_type=recent&count=100&q=%23%E7%AC%A6%E5%8D%8E%20OR%20%23%E5%B4%A9%E5%9D%8F3%20OR%20%23%E3%83%95%E3%82%AB%20OR%20%23%E5%B4%A9%E5%9D%8F3rd%20OR%20%23%E5%B4%A9%E5%A3%9E3rd%20OR%20%23%EB%B6%95%EA%B4%B43rd%20OR%20%23Honkaiimpact3rd%20OR%20%23%E5%B4%A9%E5%A3%8A3rd%20min_faves%3A2"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::fs;

    const SAMPLE_JSON_PATH: &str = "tests/assets/json/twitter-honkai.json";

    #[test]
    fn test_from_twitter_response_to_vec_posts() {
        let sample_json = fs::read_to_string(SAMPLE_JSON_PATH).expect("Unable to read the file");
        let jd = &mut serde_json::Deserializer::from_str(&sample_json);
        let result: Result<TwitterHonkaiResponse, _> = serde_path_to_error::deserialize(jd);

        result.unwrap();
    }
}
