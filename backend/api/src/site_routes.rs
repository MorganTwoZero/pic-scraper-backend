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

pub(crate) async fn like(
    State(ApiState { api_client, .. }): State<ApiState>,
    Query(PostLink { post_link }): Query<PostLink>,
) -> Result<(), Error> {
    api_client.post(make_like_url(post_link)).send().await?;
    Ok(())
}

fn make_like_url(post_link: String) -> String {
    let index = post_link.len()
        - post_link
            .chars()
            .take(19)
            .map(|c| c.len_utf8())
            .sum::<usize>();
    let post_id = &post_link[index..];
    "https://api.twitter.com/1.1/favorites/create.json?id=".to_string() + post_id
}

#[cfg(test)]
mod tests {
    use super::make_like_url;

    #[test]
    fn test_make_like_url() {
        let sample_link = "https://twitter.com/magion02/status/1647973748564455425".to_string();
        let like_url = make_like_url(sample_link);
        assert_eq!(
            "https://api.twitter.com/1.1/favorites/create.json?id=1647973748564455425",
            like_url
        );
    }
}