use axum::body::{Bytes, StreamBody};
use axum::extract::{Query, State};
use futures_util::stream::Stream;
use reqwest::Client;

use crate::startup::AppState;
use crate::Error;

#[derive(serde::Deserialize)]
pub struct Url {
    url: String,
}

pub async fn proxy_image_route(
    State(AppState { api_client, .. }): State<AppState>,
    url: Query<Url>,
) -> Result<StreamBody<impl Stream<Item = reqwest::Result<Bytes>>>, Error> {
    let res = if url.url.contains("imageView") {
        api_client.get(&url.url).header("Referer", "").send().await?.bytes_stream()
    } else {
        api_client.get(&url.url).send().await?.bytes_stream()
    };
    Ok(StreamBody::new(res))
}

pub async fn proxy_image(
    url: &str,
    client: &Client,
) -> Result<StreamBody<impl Stream<Item = reqwest::Result<Bytes>>>, Error> {
    let res = client.get(url).send().await?.bytes_stream();
    Ok(StreamBody::new(res))
}

