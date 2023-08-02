use chrono::DateTime;
use serde::Deserialize;

use crate::{
    transform::{Post, PostSource},
    Error,
};

use super::DataSource;

#[derive(Deserialize)]
pub struct TwitterHomeResponse(Vec<TweetHome>);

#[derive(Deserialize)]
struct TweetHome {
    created_at: String,
    entities: Entities,
    user: User,
}

#[derive(Deserialize)]
struct Entities {
    hashtags: Option<Vec<HashTag>>,
    media: Option<Vec<Media>>,
}

#[derive(Deserialize)]
struct HashTag {
    text: String,
}

#[derive(Deserialize)]
struct Media {
    media_url_https: String,
    expanded_url: String,
}

#[derive(Deserialize)]
struct User {
    name: String,
    screen_name: String,
    profile_image_url_https: String,
}

impl TryFrom<TweetHome> for Post {
    type Error = Error;

    fn try_from(value: TweetHome) -> Result<Self, Self::Error> {
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
            source: PostSource::TwitterHome,
            images_number: media.len() as i32,
            tags,
            author_profile_image: Some(value.user.profile_image_url_https),
        })
    }
}

impl From<TwitterHomeResponse> for Vec<Post> {
    fn from(value: TwitterHomeResponse) -> Self {
        value
            .0
            .into_iter()
            .filter(|tw| tw.entities.media.is_some())
            .filter_map(|tw| Post::try_from(tw).ok())
            .collect()
    }
}

impl DataSource for TwitterHomeResponse {
    fn url() -> &'static str {
        "https://api.twitter.com/1.1/statuses/home_timeline.json?tweet_mode=extended&exclude_replies=1&include_rts=0&count=200"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::fs;

    const SAMPLE_JSON_PATH: &str = "../tests/assets/json/twitter-home.json";

    #[test]
    fn test_from_twitter_home_response_to_vec_posts() {
        let sample_json = fs::read_to_string(SAMPLE_JSON_PATH).expect("Unable to read the file");
        serde_json::from_str::<TwitterHomeResponse>(&sample_json).unwrap();
    }
}
