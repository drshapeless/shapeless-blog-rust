use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

use tracing::error;

pub enum WebError {
    SqlxError(sqlx::Error),
    TemplateError(tinytemplate::error::Error),
    NotFound,
}

fn server_error_response() -> (StatusCode, Html<String>) {
    let server_error_str = include_str!("templates/server_error.html");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(server_error_str.to_string()),
    )
}

fn not_found_response() -> (StatusCode, Html<String>) {
    let not_found_str = include_str!("templates/not_found.html");
    (StatusCode::NOT_FOUND, Html(not_found_str.to_string()))
}

impl IntoResponse for WebError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::SqlxError(err) => match err {
                sqlx::Error::RowNotFound => not_found_response().into_response(),
                _ => {
                    error!("{:?}", err);
                    server_error_response().into_response()
                }
            },
            Self::TemplateError(err) => {
                error!("{:?}", err);
                server_error_response().into_response()
            }
            Self::NotFound => not_found_response().into_response(),
        }
    }
}
