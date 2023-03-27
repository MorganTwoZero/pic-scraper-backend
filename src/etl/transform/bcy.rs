use chrono::{TimeZone, Utc};
use serde::Deserialize;

use crate::etl::transform::{Post, PostSource};

use super::DataSource;

#[derive(Deserialize)]
pub struct BcyResponse {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    item_detail: ItemDetail,
}

#[derive(Deserialize)]
struct ItemDetail {
    item_id: String,
    uid: i64,
    uname: String,
    avatar: String,
    ctime: i64,
    cover: Option<String>,
    pic_num: Option<i32>,
}

impl From<ItemDetail> for Post {
    fn from(value: ItemDetail) -> Self {
        let created = Utc
            .timestamp_opt(value.ctime.try_into().unwrap(), 0)
            .unwrap()
            .to_rfc3339()
            .split_once("+")
            .unwrap()
            .0
            .to_string();
        Self {
            preview_link: value.cover.unwrap(),
            post_link: format!("https://bcy.net/item/detail/{}", value.item_id),
            author_link: format!("https://bcy.net/u/{}", value.uid),
            author: value.uname,
            created,
            source: PostSource::Bcy,
            images_number: value.pic_num.expect("Checked in From<BcyResponse>"),
            tags: None,
            author_profile_image: Some(value.avatar),
        }
    }
}

impl From<BcyResponse> for Vec<Post> {
    fn from(value: BcyResponse) -> Self {
        value
            .data
            .items
            .into_iter()
            .filter(|item| item.item_detail.pic_num.is_some())
            .filter(|item| item.item_detail.cover.is_some())
            .map(|item| Post::from(item.item_detail))
            .collect()
    }
}

impl DataSource for BcyResponse {
    fn url() -> &'static str {
        "https://bcy.net/apiv3/common/circleFeed?circle_id=109315&since=0&sort_type=2&grid_type=10"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::fs;

    const SAMPLE_JSON_PATH: &str = "tests/test-json/bcy.json";

    #[test]
    fn test_from_bcy_response_to_vec_posts() {
        let sample_json = fs::read_to_string(SAMPLE_JSON_PATH).expect("Unable to read the file");
        serde_json::from_str::<BcyResponse>(&sample_json).unwrap();
    }    
}