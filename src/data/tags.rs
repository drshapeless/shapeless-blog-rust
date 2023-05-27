use super::{
    blogs::{self, SimpleBlog},
    Result,
};
use serde::Serialize;
use sqlx::{PgConnection, Postgres};

#[derive(sqlx::FromRow, Serialize)]
pub struct Tag {
    name: String,
    blog_id: i64,
}

async fn _create_tag(tag: Tag, conn: &mut PgConnection) -> Result<Tag> {
    // It probably does not have to return a tag.
    let q = "
INSERT INTO tags (name, blog_id)
VALUES ($1, $2)
RETURNING *";

    let t = sqlx::query_as::<_, Tag>(q)
        .bind(tag.name)
        .bind(tag.blog_id)
        .fetch_one(conn)
        .await?;

    Ok(t)
}

pub async fn create_some_tags(
    tags: &[String],
    blog_id: i64,
    conn: &mut PgConnection,
) -> Result<()> {
    // Returns nothing when success
    let mut query_builder: sqlx::QueryBuilder<Postgres> =
        sqlx::query_builder::QueryBuilder::new("INSERT INTO tags(name, blog_id)");

    query_builder.push_values(tags.iter(), |mut b, tag| {
        b.push_bind(tag).push_bind(blog_id);
    });

    let query = query_builder.build();

    query.execute(conn).await?;

    Ok(())
}

pub async fn _get_tags_by_blog_id(id: i64, conn: &mut PgConnection) -> Result<Vec<Tag>> {
    let q = "
SELECT * FROM tags
WHERE blog_id = $1";

    let tags = sqlx::query_as::<_, Tag>(q).bind(id).fetch_all(conn).await?;

    Ok(tags)
}

async fn get_blog_ids_by_tag_name(name: String, conn: &mut PgConnection) -> Result<Vec<i64>> {
    let blog_ids = sqlx::query!(
        "
SELECT id
FROM blogs
JOIN tags ON blogs.id = tags.blog_id
WHERE tags.name = $1
ORDER BY id DESC",
        name
    )
    .fetch_all(conn)
    .await?;

    let mut ids: Vec<i64> = Vec::new();

    for blog_id in blog_ids {
        ids.push(blog_id.id);
    }

    Ok(ids)
}

pub async fn get_simple_blogs_by_tag_name(
    name: String,
    conn: &mut PgConnection,
) -> Result<Vec<SimpleBlog>> {
    let blog_ids = get_blog_ids_by_tag_name(name, conn).await?;

    let mut blogs: Vec<SimpleBlog> = Vec::new();

    for id in blog_ids {
        let blog = blogs::get_simple_blog(id, conn).await?;
        blogs.push(blog);
    }

    Ok(blogs)
}

pub async fn delete_all_tags_for_blog_id(blog_id: i64, conn: &mut PgConnection) -> Result<bool> {
    let q = "
DELETE FROM tags
WHERE blog_id = $1";

    let result = sqlx::query(q).bind(blog_id).execute(conn).await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_all_tag_names(conn: &mut PgConnection) -> Result<Vec<String>> {
    let result = sqlx::query!("SELECT DISTINCT name FROM tags ORDER BY name ASC")
        .fetch_all(conn)
        .await?;

    let mut names: Vec<String> = Vec::new();

    for r in result {
        names.push(r.name);
    }

    Ok(names)
}
