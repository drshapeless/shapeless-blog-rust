use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, patch, post, put};
use axum::{extract::State, Router};
use axum::{middleware, Extension, Json};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::data::{blogs, tags};

use super::errors::ApiError;
use super::helpers::{get_conn_from_pool, get_tx_from_pool};

use super::middlewares::auth;
use super::Result;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/blog/", post(create_blog_handler))
        .route(
            "/blog/:id",
            patch(update_blog_handler)
                .delete(delete_blog_handler)
                .get(show_blog_handler),
        )
        .route("/blogs/", get(show_all_simple_blogs_handler))
        .route("/force-blog/", post(force_create_blog_handler))
        .route("/force-blog/:id", put(force_update_blog_handler))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state)
}

#[derive(Serialize, Deserialize)]
struct NewBlogWithTags {
    url: String,
    title: String,
    preview: String,
    content: String,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct UpdatedBlogWithTags {
    url: Option<String>,
    title: Option<String>,
    preview: Option<String>,
    content: Option<String>,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ForceFullBlog {
    url: String,
    title: String,
    preview: String,
    content: String,
    create_time: String,
    edit_time: String,
    tags: Vec<String>,
}

async fn create_blog_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(new_blog_with_tags): Json<NewBlogWithTags>,
) -> Result<(StatusCode, Json<blogs::FullBlog>)> {
    // Here we must use begin.
    // If any part of the insertions failed, discard all the changes.
    // We only need to commit the successful changes.
    // If the transcation goes out of scope, the rollback will
    // automatically be called.
    let mut tx = get_tx_from_pool(state.db).await?;

    let new_blog = blogs::NewBlog::new(
        user_id,
        new_blog_with_tags.url,
        new_blog_with_tags.title,
        new_blog_with_tags.preview,
        new_blog_with_tags.content,
    );

    let blog = blogs::create_blog(new_blog, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    let tags = new_blog_with_tags.tags;
    tags::create_some_tags(&tags, blog.id, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    tx.commit().await.map_err(ApiError::SqlxError)?;

    Ok((
        StatusCode::CREATED,
        Json(blogs::FullBlog::from_blog_and_tags(blog, tags)),
    ))
}

async fn update_blog_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(id): Path<i64>,
    Json(updated_blog_with_tags): Json<UpdatedBlogWithTags>,
) -> Result<StatusCode> {
    if id < 0 {
        return Err(ApiError::NotFound);
    }

    let mut tx = get_tx_from_pool(state.db).await?;

    let mut blog = blogs::get_blog(id, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    if blog.user_id != user_id {
        return Err(ApiError::Unauthorized);
    }

    blog.url = updated_blog_with_tags.url.unwrap_or(blog.url);
    blog.title = updated_blog_with_tags.title.unwrap_or(blog.title);
    blog.preview = updated_blog_with_tags.preview.unwrap_or(blog.preview);
    blog.content = updated_blog_with_tags.content.unwrap_or(blog.content);

    let _blog = blogs::update_blog(blog, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    if let Some(tags) = updated_blog_with_tags.tags {
        // If new tags are found, delete the original tags.
        tags::delete_all_tags_for_blog_id(id, &mut tx)
            .await
            .map_err(ApiError::SqlxError)?;

        tags::create_some_tags(&tags, id, &mut tx)
            .await
            .map_err(ApiError::SqlxError)?;
    }

    tx.commit().await.map_err(ApiError::SqlxError)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn show_blog_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<blogs::FullBlog>> {
    if id < 0 {
        return Err(ApiError::NotFound);
    }

    let mut conn = get_conn_from_pool(state.db).await?;

    let blog = blogs::get_full_blog(id, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    Ok(Json(blog))
}

async fn delete_blog_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    if id < 0 {
        return Err(ApiError::NotFound);
    }

    let mut conn = get_conn_from_pool(state.db).await?;

    let blog_user_id = blogs::get_user_id_with_blog_id(id, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    if blog_user_id != user_id {
        return Err(ApiError::Unauthorized);
    }

    let success = blogs::delete_blog(id, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    if success {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

async fn show_all_simple_blogs_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<blogs::SimpleBlog>>> {
    let mut conn = get_conn_from_pool(state.db).await?;

    let blogs = blogs::get_all_simple_blogs(&mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    Ok(Json(blogs))
}

fn parse_time_string(str: String) -> Result<DateTime<Utc>> {
    let naive_date = NaiveDate::parse_from_str(str.as_str(), "%Y-%m-%d")
        .map_err(ApiError::InvalidTimeString)?
        .and_hms_opt(0, 0, 0);

    match naive_date {
        Some(d) => Ok(DateTime::<Utc>::from_utc(d, Utc)),
        None => Err(ApiError::InternalServerError(
            "datetime conversion error".to_string(),
        )),
    }
}

async fn force_create_blog_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(new_blog_with_tags): Json<ForceFullBlog>,
) -> Result<(StatusCode, Json<blogs::FullBlog>)> {
    let mut tx = get_tx_from_pool(state.db).await?;

    let create_time = parse_time_string(new_blog_with_tags.create_time)?;

    let edit_time = parse_time_string(new_blog_with_tags.edit_time)?;

    let new_blog = blogs::ForceNewBlog::new(
        user_id,
        new_blog_with_tags.url,
        new_blog_with_tags.title,
        new_blog_with_tags.preview,
        new_blog_with_tags.content,
        create_time,
        edit_time,
    );

    let blog = blogs::force_create_blog(new_blog, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    let tags = new_blog_with_tags.tags;
    tags::create_some_tags(&tags, blog.id, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    tx.commit().await.map_err(ApiError::SqlxError)?;

    Ok((
        StatusCode::CREATED,
        Json(blogs::FullBlog::from_blog_and_tags(blog, tags)),
    ))
}

async fn force_update_blog_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(id): Path<i64>,
    Json(updated_blog_with_tags): Json<ForceFullBlog>,
) -> Result<StatusCode> {
    if id < 0 {
        return Err(ApiError::NotFound);
    }

    let mut tx = get_tx_from_pool(state.db).await?;

    let mut blog = blogs::get_blog(id, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    if blog.user_id != user_id {
        return Err(ApiError::Unauthorized);
    }

    blog.url = updated_blog_with_tags.url;
    blog.title = updated_blog_with_tags.title;
    blog.preview = updated_blog_with_tags.preview;
    blog.content = updated_blog_with_tags.content;
    blog.create_time = parse_time_string(updated_blog_with_tags.create_time)?;
    blog.edit_time = parse_time_string(updated_blog_with_tags.edit_time)?;

    let _blog = blogs::force_update_blog(blog, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    tags::delete_all_tags_for_blog_id(id, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    tags::create_some_tags(&updated_blog_with_tags.tags, id, &mut tx)
        .await
        .map_err(ApiError::SqlxError)?;

    tx.commit().await.map_err(ApiError::SqlxError)?;

    Ok(StatusCode::NO_CONTENT)
}
