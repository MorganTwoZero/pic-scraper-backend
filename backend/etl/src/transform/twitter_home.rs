use async_trait::async_trait;
use chrono::DateTime;
use serde::{Deserialize, Deserializer};

use crate::{
    transform::{Post, PostSource},
    Error,
};

use super::DataSource;

#[derive(Deserialize)]
pub struct TwitterHomeResponse {
    pub data: Data,
}

#[derive(Deserialize)]
pub struct Data {
    pub home: Home,
}

#[derive(Deserialize)]
pub struct Home {
    pub home_timeline_urt: HomeTimelineUrt,
}

#[derive(Deserialize)]
pub struct HomeTimelineUrt {
    pub instructions: Vec<Instruction>,
}

#[derive(Deserialize)]
pub struct Instruction {
    pub entries: Vec<Entry>,
}

pub struct Entry {
    pub entry_id: String,
    pub content: Option<Content>,
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json: serde_json::Value = serde_json::Value::deserialize(deserializer)?;
        let entry_id = json["entryId"].as_str().unwrap().to_owned();

        let content = match entry_id.starts_with("tweet") {
            true => Some(serde_json::from_value::<Content>(json["content"].clone()).unwrap()),
            false => None,
        };

        Ok(Entry { entry_id, content })
    }
}

#[derive(Deserialize)]
pub struct Content {
    #[serde(rename = "itemContent")]
    pub item_content: ItemContent,
}

#[derive(Deserialize)]
pub struct ItemContent {
    pub tweet_results: TweetResults,
}

#[derive(Deserialize)]
pub struct TweetResults {
    pub result: TweetResult,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum TweetResult {
    Limited(Limited),
    Normal(Tweet),
}

#[derive(Deserialize)]
pub struct Limited {
    pub tweet: Tweet,
}

#[derive(Deserialize)]
pub struct Tweet {
    pub core: Core,
    pub legacy: Legacy2,
}

#[derive(Deserialize)]
pub struct Core {
    pub user_results: UserResults,
}

#[derive(Deserialize)]
pub struct UserResults {
    pub result: Result2,
}

#[derive(Deserialize)]
pub struct Result2 {
    pub legacy: Legacy,
}

#[derive(Deserialize)]
pub struct Legacy {
    pub name: String,
    pub profile_image_url_https: String,
    pub screen_name: String,
}

#[derive(Deserialize)]
pub struct Legacy2 {
    pub created_at: String,
    pub entities: Entities,
}

#[derive(Deserialize)]
pub struct Entities {
    pub media: Option<Vec<Media>>,
}

#[derive(Deserialize)]
pub struct Media {
    pub expanded_url: String,
    pub media_url_https: String,
}

impl TryFrom<Tweet> for Post {
    type Error = Error;

    fn try_from(value: Tweet) -> Result<Self, Self::Error> {
        let user = value.core.user_results.result.legacy;
        let value = value.legacy;
        let created = DateTime::parse_from_str(&value.created_at, "%a %b %d %H:%M:%S %z %Y")
            .ok()
            .ok_or(Error::Parsing)?
            .to_rfc3339();
        let media = value
            .entities
            .media
            .expect("Checked in From<TwitterHomeResponse>");
        let main_pic = <Vec<Media> as AsRef<[Media]>>::as_ref(media.as_ref())
            .first()
            .ok_or(Error::Parsing)?;
        Ok(Self {
            preview_link: main_pic.media_url_https.to_string(),
            post_link: main_pic.expanded_url.replace("/photo/1", ""),
            author_link: format!("https://twitter.com/{}", user.screen_name),
            author: format!("{}@{}", user.name, user.screen_name),
            created,
            source: PostSource::TwitterHome,
            images_number: media.len() as i32,
            tags: None,
            author_profile_image: Some(user.profile_image_url_https),
        })
    }
}

impl From<TwitterHomeResponse> for Vec<Post> {
    fn from(value: TwitterHomeResponse) -> Self {
        value
            .data
            .home
            .home_timeline_urt
            .instructions
            .into_iter()
            .flat_map(|instruction| {
                instruction
                    .entries
                    .into_iter()
                    .filter_map(|entry| match entry.content {
                        Some(content) if entry.content.is_some() => {
                            let tweet = match content.item_content.tweet_results.result {
                                TweetResult::Normal(normal) => normal,
                                TweetResult::Limited(limited) => limited.tweet,
                            };
                            if tweet.legacy.entities.media.is_some() {
                                Post::try_from(tweet).ok()
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
            })
            .collect()
    }
}

#[async_trait]
impl DataSource for TwitterHomeResponse {}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    const SAMPLE_JSON_PATH: &str = "../tests/assets/json/twitter-home.json";

    #[test]
    fn test_twitter_home_from_json() {
        let sample_json = fs::read_to_string(SAMPLE_JSON_PATH).expect("Unable to read the file");
        let jd = &mut serde_json::Deserializer::from_str(&sample_json);
        let result: Result<TwitterHomeResponse, _> = serde_path_to_error::deserialize(jd);

        result.unwrap();
    }
}
