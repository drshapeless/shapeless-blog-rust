use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{config::Config, rest, web};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config) -> Self {
        Self {
            db,
            config: Arc::new(config),
        }
    }
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .merge(rest::routes(state.clone()))
        .merge(web::routes(state))
}
