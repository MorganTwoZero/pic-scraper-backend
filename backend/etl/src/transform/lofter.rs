use async_trait::async_trait;
use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;

use crate::transform::{DataSource, MultiUrlDataSource, Post, PostSource};
use crate::Error;

#[derive(Deserialize)]
pub struct LofterResponse {
    pub items: Vec<LofterPost>,
}

#[derive(Deserialize)]
pub struct Resp {}

#[derive(Deserialize)]
pub struct LofterPost {
    pub url: String,
    pub content_html: String,
    pub date_published: String,
    pub authors: Vec<Author>,
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct Author {
    pub name: String,
}

impl TryFrom<LofterPost> for Post {
    fn try_from(value: LofterPost) -> Result<Self, Error> {
        Ok(Self {
            post_link: value.url.to_owned(),
            author_link: value
                .url
                .split("/post")
                .next()
                .ok_or(Error::Parsing)?
                .to_owned(),
            preview_link: extract_preview_img(&value.content_html)
                .ok_or(Error::Parsing)?
                .to_owned(),
            author: value.authors.into_iter().next().ok_or(Error::Parsing)?.name,
            created: value.date_published,
            source: PostSource::Lofter,
            images_number: 1,
            tags: Some(value.tags),
            author_profile_image: None,
        })
    }

    type Error = Error;
}

impl From<LofterResponse> for Vec<Post> {
    fn from(value: LofterResponse) -> Self {
        value
            .items
            .into_iter()
            .filter_map(|p| Post::try_from(p).ok())
            .collect()
    }
}

#[derive(Debug, Deserialize)]
enum Tag {
    崩坏3,
    符华,
    琪亚娜,
    丽塔,
    崩坏三,
    崩坏3rd,
    雷电芽衣,
}

impl DataSource for LofterResponse {
    fn url() -> &'static str {
        "https://rsshub.app/lofter/tag/"
    }
}

#[async_trait]
impl MultiUrlDataSource for LofterResponse {
    // TODO
    fn tags(base_url: &str) -> Vec<String> {
        let tags = vec![
            Tag::崩坏3,
            Tag::符华,
            Tag::琪亚娜,
            Tag::丽塔,
            Tag::崩坏三,
            Tag::崩坏3rd,
            Tag::雷电芽衣,
        ];
        tags.into_iter()
            .map(|tag| {
                format!(
                    "{}{:?}/date?format=json&filter_description=img",
                    base_url, tag
                )
            })
            .collect()
    }

    async fn request_and_parse_multi(
        client: &Client,
        urls: Vec<String>,
    ) -> Result<Vec<Post>, Error> {
        let requests = urls.into_iter().map(|url| fetch_url(client, url));

        let responses = join_all(requests).await;
        let mut posts = vec![];
        for res in responses {
            match res {
                Ok(res) => posts.push(<LofterResponse as Into<Vec<Post>>>::into(res)),
                Err(e) => tracing::error!("{:?}", e),
            }
        }
        Ok(posts.into_iter().flatten().collect())
    }
}

#[tracing::instrument(skip(client), level = "trace")]
async fn fetch_url(
    client: &reqwest::Client,
    url: String,
) -> Result<LofterResponse, reqwest::Error> {
    client.get(url).send().await?.json::<LofterResponse>().await
}

/// all blame goes to LLMs
fn extract_preview_img(input_str: &str) -> Option<&str> {
    let start_idx = input_str.find("src=\"").map(|i| i + 5).unwrap_or(0);
    let end_idx = input_str[start_idx..]
        .find('\"')
        .map(|i| i + start_idx)
        .unwrap_or(0);

    if start_idx != 0 && end_idx != 0 {
        Some(&input_str[start_idx..end_idx])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const LOFTER_JSON_PATH: &str = "../tests/assets/json/lofter.json";

    #[test]
    fn test_from_lofter_response_to_vec_posts() {
        let sample_json = fs::read_to_string(LOFTER_JSON_PATH).expect("Unable to read the file");
        let jd = &mut serde_json::Deserializer::from_str(&sample_json);
        let result: Result<LofterResponse, _> = serde_path_to_error::deserialize(jd);

        result.unwrap();
    }

    #[test]
    fn test_extract_preview_img() {
        let input_str = r#"<img src="https://imglf5.lf127.net/img/a90519c4da131d98/blZyZHNVUENMRnJFbXRzSlZZSHpCTWh3RnBNMnNFWHZ0VU5nRHpiTG04ST0.png?imageView&amp;thumbnail=1680x0&amp;quality=96&amp;stripmeta=0" referrerpolicy="no-referrer"><img src="https://imglf4.lf127.net/img/7012891e43c59bf3/blZyZHNVUENMRnJFbXRzSlZZSHpCSEJuV1FxZExGRS84QkRGK2U5dU41QT0.gif" referrerpolicy="no-referrer"><p id="p_u5fltfueecg">画了。不想抠细节，饶了我吧。</p> \n<p id="p_uleltfueech">很喜欢高短马尾，有种将军的感觉，四舍五入圆了戍边梦。</p> \n<p id="p_unjltgvzvlz">二编：把签名去了因为实在太丑，加了个过程gif（有点像什么羞耻play</p>"#;
        let output = extract_preview_img(input_str).unwrap();
        assert_eq!(output, "https://imglf5.lf127.net/img/a90519c4da131d98/blZyZHNVUENMRnJFbXRzSlZZSHpCTWh3RnBNMnNFWHZ0VU5nRHpiTG04ST0.png?imageView&amp;thumbnail=1680x0&amp;quality=96&amp;stripmeta=0")
    }
}
