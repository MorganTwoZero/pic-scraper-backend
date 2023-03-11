use std::collections::HashMap;

use chrono::DateTime;
use serde::{Deserialize, Serialize};

use crate::etl::transform::{Post, PostSource};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwitterResponse {
    global_objects: GlobalObjects,
}

#[derive(Serialize, Deserialize)]
struct GlobalObjects {
    tweets: HashMap<String, Tweet>,
}

#[derive(Serialize, Deserialize)]
pub struct Tweet {
    id: i128,
    full_text: String,
    entities: Entities,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct Entities {
    media: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize)]
struct Media {
    expanded_url: String,
    media_url_https: String,
}

impl TryFrom<Tweet> for Post {
    type Error = String;

    fn try_from(value: Tweet) -> Result<Self, Self::Error> {
        if value.entities.media.is_none() {
            Err("Not an image".to_string())
        } else {
            let created = DateTime::parse_from_str(&value.created_at, "%a %b %d %H:%M:%S %z %Y")
                .unwrap()
                .to_rfc3339()
                .split_once("+")
                .unwrap()
                .0
                .to_string();
            Ok (Self {
                preview_link: value
                    .entities
                    .media
                    .as_ref()
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .media_url_https
                    .to_owned(),
                post_link: value
                    .entities
                    .media
                    .as_ref()
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .expanded_url
                    .replace("/photo/1", ""),
                author_link: value
                    .entities
                    .media
                    .as_ref()
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .expanded_url
                    .rsplit_once("/status/")
                    .unwrap()
                    .0
                    .to_string(),
                author: value
                    .entities
                    .media
                    .as_ref()
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .expanded_url
                    .rsplit_once("/status")
                    .unwrap()
                    .0
                    .rsplit_once("twitter.com/")
                    .unwrap()
                    .1
                    .to_string(),
                created,
                source: PostSource::Twitter,
                images_number: value.entities.media.unwrap().len() as i32,
            })
        }
    }
}

impl From<TwitterResponse> for Vec<Post> {
    fn from(mut value: TwitterResponse) -> Self {
        value
            .global_objects
            .tweets
            .drain()
            .filter_map(|tw| Post::try_from(tw.1).ok())
            .collect()
    }
}
