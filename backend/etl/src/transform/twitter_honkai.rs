use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::{
    transform::{Post, PostSource},
    Error,
};

use super::DataSource;

#[derive(Serialize, Deserialize)]
struct TweetResult {
    #[serde(rename = "entryId")]
    entry_id: String,
    content: TweetContent,
}

#[derive(Serialize, Deserialize)]
struct TweetContent {
    #[serde(rename = "itemContent")]
    item_content: ItemContent,
}

#[derive(Serialize, Deserialize)]
struct ItemContent {
    tweet_results: TweetResults,
}

#[derive(Serialize, Deserialize)]
struct TweetResults {
    result: Tweet,
}

#[derive(Serialize, Deserialize)]
struct Tweet {
    core: Core,
    legacy: Legacy,
}

#[derive(Serialize, Deserialize)]
struct Core {
    user_results: UserResults,
}

#[derive(Serialize, Deserialize)]
struct UserResults {
    result: User,
}

#[derive(Serialize, Deserialize, Clone)]
struct User {
    legacy: UserLegacy,
}

#[derive(Serialize, Deserialize, Clone)]
struct UserLegacy {
    name: String,
    profile_image_url_https: String,
    screen_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Legacy {
    created_at: String,
    entities: Entities,
}

#[derive(Serialize, Deserialize, Clone)]
struct Entities {
    media: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Media {
    expanded_url: String,
    media_url_https: String,
}

#[derive(Serialize, Deserialize)]
struct Cursor {
    #[serde(rename = "entryId")]
    entry_id: String,
    content: CursorContent,
}

#[derive(Serialize, Deserialize)]
struct CursorContent {
    #[serde(rename = "entryType")]
    entry_type: String,
    value: String,
    #[serde(rename = "cursorType")]
    cursor_type: String,
}

#[derive(Serialize, Deserialize)]
struct Data {
    search_by_raw_query: SearchByRawQuery,
}

#[derive(Serialize, Deserialize)]
pub struct TwitterHonkaiResponse {
    data: Data,
}

#[derive(Serialize, Deserialize)]
struct SearchByRawQuery {
    search_timeline: SearchTimeline,
}

#[derive(Serialize, Deserialize)]
struct SearchTimeline {
    timeline: Timeline,
}

#[derive(Serialize, Deserialize)]
struct Timeline {
    instructions: Vec<Instruction>,
}

#[derive(Serialize, Deserialize)]
struct Instruction {
    entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Entry {
    TweetResult(TweetResult),
    Cursor(Cursor),
}

impl TryFrom<TweetResult> for Post {
    type Error = Error;

    fn try_from(value: TweetResult) -> Result<Self, Self::Error> {
        let user = value
            .content
            .item_content
            .tweet_results
            .result
            .core
            .user_results
            .result
            .legacy
            .clone();
        let value = value
            .content
            .item_content
            .tweet_results
            .result
            .legacy
            .clone();
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
            .flat_map(|entry| {
                entry.entries.into_iter().filter_map(|entry| match entry {
                    Entry::TweetResult(tw)
                        if tw
                            .content
                            .item_content
                            .tweet_results
                            .result
                            .legacy
                            .entities
                            .media
                            .is_some() =>
                    {
                        Post::try_from(tw).ok()
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
