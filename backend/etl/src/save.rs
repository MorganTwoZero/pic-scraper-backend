use sqlx::PgPool;

use crate::{
    transform::{Post, PostSource},
    Result,
};

pub async fn save_honkai_posts(db_pool: &PgPool, posts: Vec<Post>) -> Result<()> {
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
pub async fn save_update_time(db_pool: &PgPool) -> Result<()> {
    sqlx::query!(
        "UPDATE last_update_time
        SET last_update_time = NOW()
        WHERE id = 0;"
    )
    .execute(db_pool)
    .await?;
    Ok(())
}
