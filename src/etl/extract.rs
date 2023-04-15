use reqwest::Client;

use crate::config::{BlackList, SourcesUrls};
use crate::etl::load::save_honkai_posts;
use crate::etl::transform::{
    BcyResponse, DataSource, LofterResponse, MihoyoResponse, MultiUrlDataSource, PixivResponse,
    Post, TwitterHomeResponse, TwitterHonkaiResponse,
};
use crate::startup::AppState;
use crate::{Error, Result};

pub async fn create_vec_posts(
    client: &Client,
    blacklist: &BlackList,
    urls: &SourcesUrls,
) -> Result<Vec<Post>, Error> {
    let fut_tuple = futures::join!(
        PixivResponse::request_and_parse(client, &urls.pixiv),
        TwitterHonkaiResponse::request_and_parse(client, &urls.twitter_honkai),
        MihoyoResponse::request_and_parse(client, &urls.mihoyo),
        BcyResponse::request_and_parse(client, &urls.bcy),
        TwitterHomeResponse::request_and_parse(client, &urls.twitter_home),
        LofterResponse::request_and_parse_multi(client, LofterResponse::urls(&urls.lofter)),
    );
    Ok(vec![
        fut_tuple.0?,
        fut_tuple.1?,
        fut_tuple.2?,
        fut_tuple.3?,
        fut_tuple.4?,
        fut_tuple.5?,
    ]
    .into_iter()
    .flatten()
    .filter(|p| !is_in_blacklist(p, blacklist))
    .collect())
}

pub async fn fill_db(state: &AppState) -> Result<String> {
    let posts = create_vec_posts(&state.api_client, &state.blacklist, &state.sources_urls).await?;
    save_honkai_posts(&state.db_pool, posts).await?;
    Ok(chrono::Utc::now().to_rfc3339())
}

fn is_in_blacklist(p: &Post, blacklist: &BlackList) -> bool {
    let author_in_blacklist = blacklist.authors.contains(&p.author);
    let tag_in_blacklist = match &p.tags {
        Some(tags) => tags.iter().any(|tag| blacklist.tags.contains(tag)),
        None => false,
    };
    author_in_blacklist | tag_in_blacklist
}

#[cfg(test)]
mod tests {
    use crate::etl::transform::PostSource;

    use super::*;

    #[test]
    fn test_check_blacklist() {
        let p = Post {
            author: "Icey Tashiko".to_string(),
            author_link: "https://www.pixiv.net/en/users/59611188".to_string(),
            author_profile_image: None,
            created: "123".to_string(),
            images_number: 1,
            post_link: "https://www.pixiv.net/en/artworks/106611397".to_string(),
            preview_link: "https://www.pixiv.sbs/en/artworks/106611397.jpg".to_string(),
            source: PostSource::Pixiv,
            tags: Some(vec!["Koikatsu".to_string()]),
        };
        let blacklist = BlackList {
            authors: vec!["123".to_string()],
            tags: vec!["Koikatsu".to_string()],
        };
        assert!(is_in_blacklist(&p, &blacklist));
    }
}
