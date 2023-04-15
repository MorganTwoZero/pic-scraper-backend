use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use futures::future::join_all;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::etl::transform::{DataSource, MultiUrlDataSource, Post, PostSource};
use crate::Error;

#[derive(Deserialize)]
pub struct LofterResponse(Vec<String>);

impl From<LofterResponse> for Vec<Post> {
    fn from(html: LofterResponse) -> Self {
        let author_profile_image = Selector::parse("div.w-img > a > img").unwrap();
        let author_link_selector = Selector::parse("div.w-img > a").unwrap();
        let post_selector = Selector::parse(".m-mlist").unwrap();
        let totalnum_selector = Selector::parse(".totalnum").unwrap();
        let preview_link_selector = Selector::parse(".imgc img").unwrap();
        let post_link_selector = Selector::parse(".isayc").unwrap();
        html.0
            .into_iter()
            .filter(|html| !html.is_empty())
            .map(|html| Html::parse_document(&html))
            .map(|fragment| {
                fragment
                    .select(&post_selector)
                    .filter(|item| match item.value().attr("data-type") {
                        Some("2") => true,
                        _ => false,
                    })
                    .map(|item| {
                        let pic_num = match item.select(&totalnum_selector).next() {
                            Some(el) => el.inner_html().parse().unwrap_or(1),
                            None => 1,
                        };
                        let author_link = item
                            .select(&author_link_selector)
                            .next()
                            .unwrap()
                            .value()
                            .attr("href")
                            .unwrap()
                            .to_string();
                        let author = item
                            .select(&author_link_selector)
                            .next()
                            .unwrap()
                            .value()
                            .attr("title")
                            .unwrap()
                            .to_string();
                        let author_profile_image = item
                            .select(&author_profile_image)
                            .next()
                            .unwrap()
                            .value()
                            .attr("src")
                            .unwrap()
                            .to_string()
                            .into();
                        let preview_link = item
                            .select(&preview_link_selector)
                            .next()
                            .unwrap()
                            .value()
                            .attr("src")
                            .unwrap()
                            .to_string();
                        let post_link = item
                            .select(&post_link_selector)
                            .next()
                            .unwrap()
                            .value()
                            .attr("href")
                            .unwrap()
                            .to_string();
                        let created = Utc
                            .timestamp_millis_opt(
                                item.select(&Selector::parse(".isayc").unwrap())
                                    .next()
                                    .unwrap()
                                    .value()
                                    .attr("data-time")
                                    .unwrap()
                                    .parse()
                                    .unwrap(),
                            )
                            .unwrap()
                            .to_rfc3339();

                        Post {
                            author_link,
                            author,
                            author_profile_image,
                            preview_link,
                            created,
                            post_link,
                            images_number: pic_num,
                            source: PostSource::Lofter,
                            tags: None,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
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
        "https://www.lofter.com/tag/"
    }
}

#[async_trait]
impl MultiUrlDataSource for LofterResponse {
    fn urls(base_url: &str) -> Vec<String> {
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
            .map(|tag| format!("{}{:?}", base_url, tag))
            .collect()
    }

    async fn request_and_parse_multi(client: &Client, urls: Vec<String>) -> Result<Vec<Post>, Error> {
        let futures = urls.iter().map(|url| fetch_url(&client, url));
        let htmls = join_all(futures).await.into_iter().collect::<Result<Vec<_>, _>>()?;
        let responses = Self(htmls);
        Ok(responses.into())
    }
}

async fn fetch_url(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    let response = client.get(url).send().await?;
    let data = response.text().await?;

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const SAMPLE_JSON_PATH: &str = "tests/assets/json/lofter.htm";

    #[test]
    fn test_from_lofter_response_to_vec_posts() {
        let html = LofterResponse(vec![
            fs::read_to_string(SAMPLE_JSON_PATH).expect("Unable to read the file")
        ]);
        let author_profile_image = Selector::parse("div.w-img > a > img").unwrap();
        let author_link_selector = Selector::parse("div.w-img > a").unwrap();
        let post_selector = Selector::parse(".m-mlist").unwrap();
        let totalnum_selector = Selector::parse(".totalnum").unwrap();
        let preview_link_selector = Selector::parse(".imgc img").unwrap();
        let post_link_selector = Selector::parse(".isayc").unwrap();
        html.0
            .into_iter()
            .filter(|html| !html.is_empty())
            .map(|html| Html::parse_document(&html))
            .map(|fragment| {
                fragment
                    .select(&post_selector)
                    .filter(|item| match item.value().attr("data-type") {
                        Some("2") => true,
                        _ => false,
                    })
                    .map(|item| {
                        let pic_num = match item.select(&totalnum_selector).next() {
                            Some(el) => el.inner_html().parse().unwrap_or(1),
                            None => 1,
                        };
                        let author_link = item
                            .select(&author_link_selector)
                            .next()
                            .unwrap()
                            .value()
                            .attr("href")
                            .unwrap()
                            .to_string();
                        let author = item
                            .select(&author_link_selector)
                            .next()
                            .unwrap()
                            .value()
                            .attr("title")
                            .unwrap()
                            .to_string();
                        let author_profile_image = item
                            .select(&author_profile_image)
                            .next()
                            .unwrap()
                            .value()
                            .attr("src")
                            .unwrap()
                            .to_string()
                            .into();
                        let preview_link = item
                            .select(&preview_link_selector)
                            .next()
                            .unwrap()
                            .value()
                            .attr("src")
                            .unwrap()
                            .chars()
                            .take(146)
                            .collect();
                        let post_link = item
                            .select(&post_link_selector)
                            .next()
                            .unwrap()
                            .value()
                            .attr("href")
                            .unwrap()
                            .to_string();
                        let created = Utc
                            .timestamp_millis_opt(
                                item.select(&Selector::parse(".isayc").unwrap())
                                    .next()
                                    .unwrap()
                                    .value()
                                    .attr("data-time")
                                    .unwrap()
                                    .parse()
                                    .unwrap(),
                            )
                            .unwrap()
                            .to_rfc3339();

                        Post {
                            author_link,
                            author,
                            author_profile_image,
                            preview_link,
                            created,
                            post_link,
                            images_number: pic_num,
                            source: PostSource::Lofter,
                            tags: None,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .for_each(drop);
    }
}
