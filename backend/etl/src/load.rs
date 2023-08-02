use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::PgPool;

use crate::transform::{Post, PostSource};
use config_structs::AppState;
use errors::Error;

#[derive(serde::Deserialize)]
pub struct Page {
    page: i64,
}

pub async fn save_honkai_posts(db_pool: &PgPool, posts: Vec<Post>) -> Result<(), Error> {
    for post in posts {
        let Post {
            post_link,
            preview_link,
            images_number,
            created,
            author,
            author_link,
            source,
            tags,
            author_profile_image,
        } = post;
        sqlx::query!(
            r#"
            INSERT INTO honkai_posts (
                post_link,
                preview_link,
                images_number,
                created,
                author,
                author_link,
                source,
                tags,
                author_profile_image
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (post_link) DO NOTHING
            "#,
            post_link,
            preview_link,
            images_number,
            created,
            author,
            author_link,
            source as PostSource,
            tags.as_deref(),
            author_profile_image.as_deref()
        )
        .execute(db_pool)
        .await?;
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn load_honkai_posts(
    State(AppState { db_pool, .. }): State<AppState>,
    page: Query<Page>,
) -> Result<Json<Vec<Post>>, Error> {
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
    State(AppState { db_pool, .. }): State<AppState>,
    page: Query<Page>,
) -> Result<Json<Vec<Post>>, Error> {
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
