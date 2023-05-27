use super::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgConnection;

#[derive(sqlx::FromRow, Serialize)]
pub struct Blog {
    pub id: i64,
    pub user_id: i64,
    pub url: String,
    pub title: String,
    pub preview: String,
    pub content: String,
    pub create_time: DateTime<Utc>,
    pub edit_time: DateTime<Utc>,
    version: i64,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct FullBlog {
    id: i64,
    pub user_id: i64,
    pub url: String,
    pub title: String,
    pub preview: String,
    pub content: String,
    pub create_time: DateTime<Utc>,
    pub edit_time: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl FullBlog {
    pub fn from_blog_and_tags(blog: Blog, tags: Vec<String>) -> Self {
        FullBlog {
            id: blog.id,
            user_id: blog.user_id,
            url: blog.url,
            title: blog.title,
            preview: blog.preview,
            content: blog.content,
            create_time: blog.create_time,
            edit_time: blog.edit_time,
            tags,
        }
    }
}

pub struct NewBlog {
    user_id: i64,
    url: String,
    title: String,
    preview: String,
    content: String,
}

impl NewBlog {
    pub fn new(user_id: i64, url: String, title: String, preview: String, content: String) -> Self {
        NewBlog {
            user_id,
            url,
            title,
            preview,
            content,
        }
    }
}

pub struct ForceNewBlog {
    user_id: i64,
    url: String,
    title: String,
    preview: String,
    content: String,
    create_time: DateTime<Utc>,
    edit_time: DateTime<Utc>,
}

impl ForceNewBlog {
    pub fn new(
        user_id: i64,
        url: String,
        title: String,
        preview: String,
        content: String,
        create_time: DateTime<Utc>,
        edit_time: DateTime<Utc>,
    ) -> Self {
        ForceNewBlog {
            user_id,
            url,
            title,
            preview,
            content,
            create_time,
            edit_time,
        }
    }
}

#[derive(sqlx::FromRow, Serialize)]
pub struct SimpleBlog {
    pub id: i64,
    pub user_id: i64,
    pub url: String,
    pub title: String,
    pub preview: String,
    pub create_time: DateTime<Utc>,
    pub edit_time: DateTime<Utc>,
    pub tags: Vec<String>,
}

pub async fn create_blog(new_blog: NewBlog, conn: &mut PgConnection) -> Result<Blog> {
    let q = "
INSERT INTO blogs (user_id, url, title, preview, content)
VALUES ($1, $2, $3, $4, $5)
RETURNING *";

    let blog = sqlx::query_as::<_, Blog>(q)
        .bind(new_blog.user_id)
        .bind(new_blog.url)
        .bind(new_blog.title)
        .bind(new_blog.preview)
        .bind(new_blog.content)
        .fetch_one(conn)
        .await?;

    Ok(blog)
}

pub async fn update_blog(updated_blog: Blog, conn: &mut PgConnection) -> Result<Blog> {
    let q = "
UPDATE blogs
SET url = $1, title = $2, preview = $3, content = $4, edit_time = NOW(), version = version + 1
WHERE id = $5 AND version = $6
RETURNING *";

    let blog = sqlx::query_as::<_, Blog>(q)
        .bind(updated_blog.url)
        .bind(updated_blog.title)
        .bind(updated_blog.preview)
        .bind(updated_blog.content)
        .bind(updated_blog.id)
        .bind(updated_blog.version)
        .fetch_one(conn)
        .await?;

    Ok(blog)
}

pub async fn get_blog(id: i64, conn: &mut PgConnection) -> Result<Blog> {
    let q = "
SELECT * FROM blogs
WHERE id = $1";

    let blog = sqlx::query_as::<_, Blog>(q)
        .bind(id)
        .fetch_one(conn)
        .await?;

    Ok(blog)
}

pub async fn get_full_blog(id: i64, conn: &mut PgConnection) -> Result<FullBlog> {
    let q = "
SELECT id, user_id, url, title, preview, content, create_time, edit_time, ARRAY_AGG(tags.name) as tags
FROM blogs
JOIN tags ON blogs.id = tags.blog_id
WHERE blogs.id = $1
GROUP BY blogs.id
";

    let blog = sqlx::query_as::<_, FullBlog>(q)
        .bind(id)
        .fetch_one(conn)
        .await?;

    Ok(blog)
}

pub async fn get_full_blog_by_url(url: String, conn: &mut PgConnection) -> Result<FullBlog> {
    let q = "
SELECT id, user_id, url, title, preview, content, create_time, edit_time, ARRAY_AGG(tags.name) as tags
FROM blogs
JOIN tags ON blogs.id = tags.blog_id
WHERE blogs.url = $1
GROUP BY blogs.id
";

    let blog = sqlx::query_as::<_, FullBlog>(q)
        .bind(url)
        .fetch_one(conn)
        .await?;

    Ok(blog)
}

pub async fn get_simple_blog(id: i64, conn: &mut PgConnection) -> Result<SimpleBlog> {
    let q = "
SELECT id, user_id, url, title, preview, create_time, edit_time, ARRAY_AGG(tags.name) as tags
FROM blogs
JOIN tags ON blogs.id = tags.blog_id
WHERE blogs.id = $1
GROUP BY blogs.id";

    let blog = sqlx::query_as::<_, SimpleBlog>(q)
        .bind(id)
        .fetch_one(conn)
        .await?;

    Ok(blog)
}

pub async fn get_all_simple_blogs(conn: &mut PgConnection) -> Result<Vec<SimpleBlog>> {
    let q = "
SELECT id, user_id, url, title, preview, create_time, edit_time, ARRAY_AGG(tags.name) as tags
FROM blogs
JOIN tags ON blogs.id = tags.blog_id
GROUP BY blogs.id
ORDER BY create_time DESC";

    let blogs = sqlx::query_as::<_, SimpleBlog>(q).fetch_all(conn).await?;

    Ok(blogs)
}

pub async fn delete_blog(id: i64, conn: &mut PgConnection) -> Result<bool> {
    let q = "
DELETE FROM blogs
WHERE id = $1";

    let result = sqlx::query(q).bind(id).execute(conn).await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_user_id_with_blog_id(blog_id: i64, conn: &mut PgConnection) -> Result<i64> {
    let user_id = sqlx::query!(
        "
SELECT user_id
FROM blogs
WHERE id = $1",
        blog_id
    )
    .fetch_one(conn)
    .await?;

    Ok(user_id.user_id)
}

pub async fn force_create_blog(blog: ForceNewBlog, conn: &mut PgConnection) -> Result<Blog> {
    let q = "
INSERT INTO blogs (user_id, url, title, preview, content, create_time, edit_time)
VALUES ($1, $2, $3, $4, $5, $6, $7)
RETURNING *";

    let blog = sqlx::query_as::<_, Blog>(q)
        .bind(blog.user_id)
        .bind(blog.url)
        .bind(blog.title)
        .bind(blog.preview)
        .bind(blog.content)
        .bind(blog.create_time)
        .bind(blog.edit_time)
        .fetch_one(conn)
        .await?;

    Ok(blog)
}

pub async fn force_update_blog(updated_blog: Blog, conn: &mut PgConnection) -> Result<Blog> {
    let q = "
UPDATE blogs
SET url = $1, title = $2, preview = $3, content = $4, create_time = $5, edit_time = $6, version = version + 1
WHERE id = $7 AND version = $8
RETURNING *";

    let blog = sqlx::query_as::<_, Blog>(q)
        .bind(updated_blog.url)
        .bind(updated_blog.title)
        .bind(updated_blog.preview)
        .bind(updated_blog.content)
        .bind(updated_blog.create_time)
        .bind(updated_blog.edit_time)
        .bind(updated_blog.id)
        .bind(updated_blog.version)
        .fetch_one(conn)
        .await?;

    Ok(blog)
}
