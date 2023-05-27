use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tinytemplate::TinyTemplate;

use crate::{
    app::AppState,
    data::{
        blogs::{self, FullBlog, SimpleBlog},
        tags::{self, get_all_tag_names},
    },
};

use super::{errors::WebError, helpers::get_conn_from_pool, Result};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(show_home_handler))
        .route("/posts/:url", get(show_blog_handler))
        .route("/tags/", get(list_tags_handler))
        .route("/tags/:name", get(show_tag_handler))
        .with_state(state)
}

#[derive(Serialize)]
struct HomeContext {
    blogs: Vec<WebSimpleBlog>,
}

#[derive(Serialize)]
struct WebSimpleBlog {
    user_id: i64,
    url: String,
    title: String,
    preview: String,
    create_time: String,
    edit_time: String,
    tags: Vec<String>,
}

impl SimpleBlog {
    fn to_web_simple_blog(&self) -> WebSimpleBlog {
        let create_time = format_datetime(self.create_time);
        let edit_time = format_datetime(self.edit_time);

        WebSimpleBlog {
            user_id: self.user_id,
            url: self.url.clone(),
            title: self.title.clone(),
            preview: self.preview.clone(),
            create_time,
            edit_time,
            tags: self.tags.clone(),
        }
    }
}

#[derive(Serialize)]
struct WebBlog {
    user_id: i64,
    url: String,
    title: String,
    preview: String,
    content: String,
    create_time: String,
    edit_time: String,
    tags: Vec<String>,
}

impl FullBlog {
    fn to_web_blog(&self) -> WebBlog {
        let create_time = format_datetime(self.create_time);
        let edit_time = format_datetime(self.edit_time);

        WebBlog {
            user_id: self.user_id,
            url: self.url.clone(),
            title: self.title.clone(),
            preview: self.preview.clone(),
            content: self.content.clone(),
            create_time,
            edit_time,
            tags: self.tags.clone(),
        }
    }
}

fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
}

fn remove_html_extension(title: String) -> String {
    let p = std::path::Path::new(title.as_str());
    let ex = match p.extension() {
        Some(extension) => extension.to_str().unwrap(),
        None => return title,
    };

    if ex == "html" {
        p.with_extension("").to_str().unwrap().to_string()
    } else {
        "".to_string()
    }
}

fn simple_blogs_to_web_simple_blogs(blogs: Vec<SimpleBlog>) -> Vec<WebSimpleBlog> {
    let mut b: Vec<WebSimpleBlog> = Vec::new();

    for blog in blogs {
        b.push(blog.to_web_simple_blog());
    }

    b
}

async fn show_home_handler(State(state): State<AppState>) -> Result {
    let home_str = include_str!("templates/home.html");

    let mut conn = get_conn_from_pool(state.db).await?;

    let blogs = blogs::get_all_simple_blogs(&mut conn)
        .await
        .map_err(WebError::SqlxError)?;

    let context = HomeContext {
        blogs: simple_blogs_to_web_simple_blogs(blogs),
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("home", home_str)
        .map_err(WebError::TemplateError)?;

    let rendered = tt
        .render("home", &context)
        .map_err(WebError::TemplateError)?;

    Ok(Html(rendered))
}

async fn show_blog_handler(State(state): State<AppState>, Path(url): Path<String>) -> Result {
    let post_str = include_str!("templates/post.html");

    if url.is_empty() {
        return Err(WebError::NotFound);
    }

    let url = remove_html_extension(url);

    let mut conn = get_conn_from_pool(state.db).await?;

    let blog = blogs::get_full_blog_by_url(url, &mut conn)
        .await
        .map_err(WebError::SqlxError)?;

    let web_blog = blog.to_web_blog();

    let mut tt = TinyTemplate::new();
    tt.set_default_formatter(&tinytemplate::format_unescaped);
    tt.add_template("post", post_str)
        .map_err(WebError::TemplateError)?;

    let rendered = tt
        .render("post", &web_blog)
        .map_err(WebError::TemplateError)?;

    Ok(Html(rendered))
}

#[derive(Serialize)]
struct TagContext {
    blogs: Vec<WebSimpleBlog>,
    tag: String,
}

async fn show_tag_handler(State(state): State<AppState>, Path(name): Path<String>) -> Result {
    let name = remove_html_extension(name);
    let tag_str = include_str!("templates/tag.html");

    let mut conn = get_conn_from_pool(state.db).await?;

    let blogs = tags::get_simple_blogs_by_tag_name(name.clone(), &mut conn)
        .await
        .map_err(WebError::SqlxError)?;

    let web_simple_blogs = simple_blogs_to_web_simple_blogs(blogs);

    let context = TagContext {
        blogs: web_simple_blogs,
        tag: name,
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("tag", tag_str)
        .map_err(WebError::TemplateError)?;

    let rendered = tt
        .render("tag", &context)
        .map_err(WebError::TemplateError)?;

    Ok(Html(rendered))
}

#[derive(Serialize)]
struct ListTagsContext {
    tags: Vec<String>,
}

async fn list_tags_handler(State(state): State<AppState>) -> Result {
    let list_tags_str = include_str!("templates/list_tags.html");

    let mut conn = get_conn_from_pool(state.db).await?;

    let tags = get_all_tag_names(&mut conn)
        .await
        .map_err(WebError::SqlxError)?;

    let context = ListTagsContext { tags };

    let mut tt = TinyTemplate::new();
    tt.add_template("list_tags", list_tags_str)
        .map_err(WebError::TemplateError)?;

    let rendered = tt
        .render("list_tags", &context)
        .map_err(WebError::TemplateError)?;

    Ok(Html(rendered))
}
