use axum::{response::Html, Router};

use crate::app::AppState;

mod blogs;
mod errors;
mod helpers;

use errors::WebError;

pub type Result<T = Html<String>, E = WebError> = std::result::Result<T, E>;

pub fn routes(state: AppState) -> Router {
    Router::new().merge(blogs::routes(state))
}
