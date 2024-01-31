use axum::extract::{Query, State};

use crate::{load::load_last_update_time, Error};
use config_structs::ApiState;

pub(crate) async fn last_update(State(state): State<ApiState>) -> Result<String, Error> {
    Ok(load_last_update_time(&state.db_pool).await?.to_rfc3339())
}

#[derive(serde::Deserialize)]
pub(crate) struct PostLink {
    post_link: String,
}

#[tracing::instrument(skip(api_client))]
pub(crate) async fn like(
    State(ApiState { api_client, .. }): State<ApiState>,
    Query(PostLink { post_link }): Query<PostLink>,
) -> Result<(), Error> {
    let json = serde_json::json!({
        "variables": { "tweet_id": extract_post_id(&post_link) }
    });
    api_client
        .post("https://twitter.com/i/api/graphql/lI07N6Otwv1PhnEgXILM7A/FavoriteTweet")
        .json(&json)
        .send()
        .await?;
    Ok(())
}

fn extract_post_id(post_link: &str) -> &str {
    let index = post_link.len()
        - post_link
            .chars()
            .take(19)
            .map(|c| c.len_utf8())
            .sum::<usize>();
    &post_link[index..]
}

#[cfg(test)]
mod tests {
    use super::extract_post_id;

    #[test]
    fn test_extract_post_id() {
        let sample_link = "https://twitter.com/magion02/status/1647973748564455425";
        let post_id = extract_post_id(sample_link);
        assert_eq!("1647973748564455425", post_id);
    }
}
