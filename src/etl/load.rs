use sqlx::PgPool;
use axum::extract::State;
use axum::Json;

use super::{Post, PostSource};

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
                source
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            post_link,
            preview_link,
            images_number,
            created,
            author,
            author_link,
            source as PostSource
        )
        .execute(db_pool)
        .await?;
    }
    Ok(())
}

pub async fn get_honkai_posts_from_db(db_pool: State<PgPool>) -> Json<Vec<Post>> {
    Json(sqlx::query_as!(
        Post,
        r#"SELECT
            post_link,
            preview_link,
            images_number,
            created,
            author,
            author_link,
            source AS "source!: PostSource"
        FROM honkai_posts"#
    )
    .fetch_all(&db_pool.0)
    .await
    .unwrap())
}
