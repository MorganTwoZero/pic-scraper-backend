use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;

use crate::Result;
use config_structs::ApiState;
use etl::{Post, PostSource};

#[derive(serde::Deserialize)]
pub struct Page {
    page: i64,
}

#[tracing::instrument(skip_all)]
pub async fn load_honkai_posts(
    State(ApiState { db_pool, .. }): State<ApiState>,
    page: Query<Page>,
) -> Result<Json<Vec<Post>>> {
    Ok(Json(
        sqlx::query_as!(
            Post,
            r#"SELECT
            post_link,
            preview_link,
            images_number,
            created,
            author,
            author_link,
            source AS "source!: PostSource",
            tags,
            author_profile_image
        FROM honkai_posts
        WHERE source != 'twitterhome'
        ORDER BY created DESC
        LIMIT 20 OFFSET $1"#,
            page.page * 20
        )
        .fetch_all(&db_pool)
        .await?,
    ))
}

#[tracing::instrument(skip_all)]
pub async fn load_twitter_home_posts(
    State(ApiState { db_pool, .. }): State<ApiState>,
    page: Query<Page>,
) -> Result<Json<Vec<Post>>> {
    Ok(Json(
        sqlx::query_as!(
            Post,
            r#"SELECT
            post_link,
            preview_link,
            images_number,
            created,
            author,
            author_link,
            source AS "source!: PostSource",
            tags,
            author_profile_image
        FROM honkai_posts
        WHERE source = 'twitterhome'
        ORDER BY created DESC
        LIMIT 20 OFFSET $1"#,
            page.page * 20
        )
        .fetch_all(&db_pool)
        .await?,
    ))
}

#[tracing::instrument(skip_all)]
pub async fn load_last_update_time(db_pool: &PgPool) -> Result<chrono::DateTime<chrono::Utc>> {
    Ok(
        sqlx::query!("SELECT last_update_time FROM last_update_time WHERE id = 0;")
            .fetch_one(db_pool)
            .await?
            .last_update_time
            .ok_or(anyhow::anyhow!("last_update_time not found"))?,
    )
}
