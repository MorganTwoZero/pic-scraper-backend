use axum::extract::{Query, State};
use axum::{headers::ContentType, TypedHeader};
use bytes::Bytes;
use serde::Deserialize;

use crate::Error;
use crate::startup::AppContext;

#[derive(Deserialize, Debug)]
pub struct Url {
    pub url: String,
}

pub async fn proxy_image(
    State(ctx): State<AppContext>,
    url: Query<Url>,
) -> Result<(TypedHeader<ContentType>, Bytes), Error> {
    let img = ctx
        .reqwest_client
        .get(&url.url)
        .send()
        .await?
        .bytes()
        .await?;
    Ok((TypedHeader(ContentType::jpeg()), img))
}
