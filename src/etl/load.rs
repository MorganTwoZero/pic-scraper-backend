use sqlx::PgPool;
use axum::extract::State;
use axum::Json;

use crate::etl::transform::{Post, PostSource};
use crate::startup::AppContext;

pub async fn save_honkai_posts(db_pool: &PgPool, posts: Vec<Post>) -> Result<(), sqlx::Error> {
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
            author_profile_image
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

pub async fn load_honkai_posts(State(ctx): State<AppContext>) -> Json<Vec<Post>> {
    let db_pool = ctx.db_pool;
    Json(sqlx::query_as!(
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
        ORDER BY created DESC"#
    )
    .fetch_all(&db_pool)
    .await
    .unwrap())
}
