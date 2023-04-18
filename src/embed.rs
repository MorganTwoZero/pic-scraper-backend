use axum::{
    async_trait,
    body::{Bytes, StreamBody},
    extract::{FromRequestParts, Query, State, TypedHeader},
    headers::UserAgent,
    http::{request::Parts, Uri},
    response::{IntoResponse, Redirect, Response},
};
use futures::Stream;
use regex::Regex;
use serde_json::Value;

use crate::{
    startup::AppState,
    utils::{proxy_image_route, Url},
    Error,
};

#[derive(Debug)]
pub struct PixivId {
    post_id: u32,
    pic_num: Option<u8>,
}

impl PixivId {
    fn full(&self) -> String {
        match self.pic_num {
            Some(num) => format!("{}_p{}", self.post_id, num),
            None => self.post_id.to_string(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for PixivId
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let re = Regex::new(r"_p|/").unwrap();

        let url_query = parts
            .uri
            .path()
            .split("en/artworks/")
            .last()
            .ok_or(Error::PixivId)?
            .trim_end_matches(".jpg");

        let mut parts = re.split(url_query);

        let post_id = parts
            .next()
            .ok_or(Error::PixivId)?
            .parse()
            .map_err(|_| Error::PixivId)?;
        let mut pic_num = parts.next().and_then(|num| num.parse().ok());
        if url_query.contains('/') {
            pic_num = pic_num.map(|v| v - 1);
        };

        Ok(Self { post_id, pic_num })
    }
}

#[tracing::instrument(skip(state))]
pub async fn embed(
    path: Uri,
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    pixiv_id: PixivId,
) -> Result<Response, Error> {
    match path.path().ends_with(".jpg") {
        true => Ok(jpg(pixiv_id, state).await?.into_response()),
        false => Ok(html(user_agent, pixiv_id).await),
    }
}

async fn html(user_agent: UserAgent, pixiv_id: PixivId) -> Response {
    const DISCORD_HEADERS: [&str; 2] = [
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 11.6; rv:92.0) Gecko/20100101 Firefox/92.0",
        "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)",
    ];

    if DISCORD_HEADERS.contains(&user_agent.as_str()) {
        let html = format!(
            r#"<meta name="twitter:card" content="summary_large_image"><meta name="twitter:image" content="https://pixiv.sbs/en/artworks/{}.jpg">"#,
            pixiv_id.full()
        );
        let mut response = html.into_response();
        response
            .headers_mut()
            .insert(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("text/html"),
            )
            .unwrap();
        response
    } else {
        Redirect::to(format!("https://www.pixiv.net/en/artworks/{}", pixiv_id.post_id).as_str())
            .into_response()
    }
}

async fn jpg(
    pixiv_id: PixivId,
    state: AppState,
) -> Result<StreamBody<impl Stream<Item = reqwest::Result<Bytes>>>, Error> {
    let url = format!("{}{}", state.sources_urls.pixiv_details, pixiv_id.post_id);

    let json = state.api_client.get(&url).send().await?;
    let post = json.json::<Value>().await?;
    let post = post
        .get("body")
        .and_then(|body| body.get("illust_details"))
        .ok_or(anyhow::anyhow!("Failed to deserialize pixiv response"))?;

    let img_url = get_img_url(
        post,
        pixiv_id.pic_num.unwrap_or(0),
        &state.sources_urls.pixiv_image,
    )
    .ok_or(anyhow::anyhow!("Failed to deserialize pixiv response"))?;
    proxy_image_route(State(state), Query(Url { url: img_url })).await
}

fn get_img_url(json: &Value, pic_num: u8, replace_str: &str) -> Option<String> {
    let img_url = if pic_num == 0 {
        json["url"].as_str()?
    } else {
        json["manga_a"][pic_num as usize]["url"].as_str()?
    };

    let re = Regex::new(r#".*/img-master"#).unwrap();
    let img_url = re.replace(img_url, replace_str).to_string();

    Some(img_url)
}
