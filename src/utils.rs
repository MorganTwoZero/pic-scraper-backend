use axum::body::{Bytes, StreamBody};
use axum::extract::{Query, State};
use futures_util::stream::Stream;

use crate::startup::AppState;
use crate::Error;

#[derive(serde::Deserialize, Debug)]
pub struct Url {
    pub(crate) url: String,
}

#[tracing::instrument(skip(api_client))]
pub async fn proxy_image_route(
    State(AppState { api_client, .. }): State<AppState>,
    url: Query<Url>,
) -> Result<StreamBody<impl Stream<Item = reqwest::Result<Bytes>>>, Error> {
    let mut req_builder = api_client.get(&url.url);
    if url.url.contains("imageView") {
        req_builder = req_builder.header("Referer", "");
    }
    let res = req_builder.send().await?.bytes_stream();
    Ok(StreamBody::new(res))
}
