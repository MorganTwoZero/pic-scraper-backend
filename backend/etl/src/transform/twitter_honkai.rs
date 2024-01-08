use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::{
    transform::{Post, PostSource},
    Error,
};

use super::DataSource;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterHonkaiResponse {
    pub data: Data,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    #[serde(rename = "search_by_raw_query")]
    pub search_by_raw_query: SearchByRawQuery,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchByRawQuery {
    #[serde(rename = "search_timeline")]
    pub search_timeline: SearchTimeline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchTimeline {
    pub timeline: Timeline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub content: Content,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "entryType")]
pub enum Content {
    #[serde(rename = "TimelineTimelineItem")]
    #[allow(non_snake_case)]
    Tweet { itemContent: ItemContent },
    #[serde(rename = "TimelineTimelineCursor")]
    Cursor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemContent {
    #[serde(rename = "tweet_results")]
    pub tweet_results: TweetResults,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweetResults {
    #[serde(rename = "result")]
    pub result: TweetResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TweetResult {
    Limited(Limited),
    Normal(Tweet),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Limited {
    pub tweet: Tweet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tweet {
    pub core: Core,
    pub legacy: TweetDetails,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Core {
    #[serde(rename = "user_results")]
    pub user_results: UserResults,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserResults {
    #[serde(rename = "result")]
    pub result: UserResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserResult {
    pub legacy: UserDetails,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDetails {
    pub name: String,
    #[serde(rename = "profile_image_url_https")]
    pub profile_image_url_https: String,
    #[serde(rename = "screen_name")]
    pub screen_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweetDetails {
    #[serde(rename = "created_at")]
    pub created_at: String,
    pub entities: Entities,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entities {
    pub media: Option<Vec<Media>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    #[serde(rename = "expanded_url")]
    pub expanded_url: String,
    #[serde(rename = "media_url_https")]
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
            .get(0)
            .ok_or(Error::Parsing)?;
        Ok(Self {
            preview_link: main_pic.media_url_https.to_string(),
            post_link: main_pic.expanded_url.replace("/photo/1", ""),
            author_link: format!("https://twitter.com/{}", user.screen_name),
            author: format!("{}@{}", user.name, user.screen_name),
            created,
            source: PostSource::Twitter,
            images_number: media.len() as i32,
            tags: None,
            author_profile_image: Some(user.profile_image_url_https),
        })
    }
}

impl From<TwitterHonkaiResponse> for Vec<Post> {
    fn from(value: TwitterHonkaiResponse) -> Self {
        value
            .data
            .search_by_raw_query
            .search_timeline
            .timeline
            .instructions
            .into_iter()
            .flat_map(|instruction| {
                instruction
                    .entries
                    .into_iter()
                    .filter_map(|entry| match entry.content {
                        Content::Tweet { itemContent } => {
                            let tweet = match itemContent.tweet_results.result {
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

    const SAMPLE_JSON_PATH: &str = "../tests/assets/json/twitter-honkai.json";

    #[test]
    fn test_from_twitter_response_to_vec_posts() {
        let sample_json = fs::read_to_string(SAMPLE_JSON_PATH).expect("Unable to read the file");
        let jd = &mut serde_json::Deserializer::from_str(&sample_json);
        let result: Result<TwitterHonkaiResponse, _> = serde_path_to_error::deserialize(jd);

        result.unwrap();
    }
}
