use std::collections::HashMap;

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::{
    etl::transform::{Post, PostSource},
    Error,
};

use super::DataSource;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterHonkaiResponse {
    global_objects: GlobalObjects,
}

#[derive(Serialize, Deserialize)]
struct GlobalObjects {
    tweets: HashMap<String, Tweet>,
}

#[derive(Serialize, Deserialize)]
pub struct Tweet {
    entities: Entities,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct Entities {
    media: Option<Vec<Media>>,
    hashtags: Option<Vec<HashTag>>,
}

#[derive(Serialize, Deserialize)]
struct HashTag {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct Media {
    expanded_url: String,
    media_url_https: String,
}

impl TryFrom<Tweet> for Post {
    type Error = Error;

    fn try_from(value: Tweet) -> Result<Self, Self::Error> {
        let created = DateTime::parse_from_str(&value.created_at, "%a %b %d %H:%M:%S %z %Y")
            .ok()
            .ok_or(Error::Parsing)?
            .to_rfc3339();
        let main_pic = value
            .entities
            .media
            .as_ref()
            .ok_or(Error::Parsing)?
            .get(0)
            .ok_or(Error::Parsing)?;
        let author_link = main_pic
            .expanded_url
            .rsplit_once("/status/")
            .ok_or(Error::Parsing)?
            .0;
        let tags = value.entities.hashtags.map(|tags| {
            tags.into_iter()
                .map(|tag| tag.text)
                .collect::<Vec<String>>()
        });
        Ok(Self {
            preview_link: main_pic.media_url_https.to_string(),
            post_link: main_pic.expanded_url.replace("/photo/1", ""),
            author_link: author_link.to_string(),
            author: author_link
                .rsplit_once("twitter.com/")
                .ok_or(Error::Parsing)?
                .1
                .to_string(),
            created,
            source: PostSource::Twitter,
            images_number: value.entities.media.ok_or(Error::Parsing)?.len() as i32,
            tags,
            author_profile_image: None,
        })
    }
}

impl From<TwitterHonkaiResponse> for Vec<Post> {
    fn from(mut value: TwitterHonkaiResponse) -> Self {
        value
            .global_objects
            .tweets
            .drain()
            .filter(|(_, tw)| tw.entities.media.is_some())
            .filter_map(|(_, tw)| Post::try_from(tw).ok())
            .collect()
    }
}

impl DataSource for TwitterHonkaiResponse {
    fn url() -> &'static str {
        "https://twitter.com/i/api/2/search/adaptive.json?include_profile_interstitial_type=1&include_blocking=1&include_blocked_by=1&include_followed_by=1&include_want_retweets=1&include_mute_edge=1&include_can_dm=1&include_can_media_tag=1&include_ext_has_nft_avatar=1&skip_status=1&cards_platform=Web-12&include_cards=1&include_ext_alt_text=true&include_quote_count=true&include_reply_count=1&tweet_mode=extended&include_entities=true&include_user_entities=true&include_ext_media_color=true&include_ext_media_availability=true&include_ext_sensitive_media_warning=true&include_ext_trusted_friends_metadata=true&send_error_codes=true&simple_quoted_tweet=true&q=%23%E7%AC%A6%E5%8D%8E%20OR%20%23%E5%B4%A9%E5%9D%8F3%20OR%20%23%E3%83%95%E3%82%AB%20OR%20%23%E5%B4%A9%E5%9D%8F3rd%20OR%20%23%E5%B4%A9%E5%A3%9E3rd%20OR%20%23%EB%B6%95%EA%B4%B43rd%20OR%20%23Honkaiimpact3rd%20OR%20%23%E5%B4%A9%E5%A3%8A3rd%20min_faves%3A2&tweet_search_mode=live&count=20&query_source=typed_query&pc=1&spelling_corrections=1&ext=mediaStats%2ChighlightedLabel%2ChasNftAvatar%2CreplyvotingDownvotePerspective%2CvoiceInfo%2Cenrichments%2CsuperFollowMetadata%2CunmentionInfo"
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
        serde_json::from_str::<TwitterHonkaiResponse>(&sample_json).unwrap();
    }
}
