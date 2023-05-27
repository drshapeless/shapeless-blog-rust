mod blogs;
mod errors;
mod helpers;
mod middlewares;
mod users;

use super::AppState;
use axum::Router;
use errors::ApiError;

pub type Result<T, E = ApiError> = std::result::Result<T, E>;

pub fn routes(state: AppState) -> Router {
    let r = Router::new()
        .merge(users::routes(state.clone()))
        .merge(blogs::routes(state));

    Router::new().nest("/api", r)
}
