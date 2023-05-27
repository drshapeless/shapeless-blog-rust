use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use tracing::{error, info};

pub enum ApiError {
    SqlxError(sqlx::error::Error),
    BcryptError(bcrypt::BcryptError),
    Unauthorized,
    NotFound,
    NoAuthorizationHeader,
    ExpiredToken,
    DuplicatedUsername(String),
    InvalidTimeString(chrono::ParseError),
    InternalServerError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_response<S: AsRef<str>>(
    status_code: StatusCode,
    error_message: S,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status_code,
        Json(ErrorResponse {
            error: error_message.as_ref().to_string(),
        }),
    )
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::SqlxError(err) => match err {
                sqlx::Error::RowNotFound => error_response(
                    StatusCode::NOT_FOUND,
                    "the requested resource could not be found",
                )
                .into_response(),
                _ => {
                    info!("{:?}", err);

                    error_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
                        .into_response()
                }
            },
            Self::BcryptError(err) => {
                info!("{:?}", err);

                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "the password cannot be verified",
                )
                .into_response()
            }
            Self::Unauthorized => error_response(
                StatusCode::UNAUTHORIZED,
                "you are not allowed to do this operation",
            )
            .into_response(),
            Self::NotFound => error_response(
                StatusCode::NOT_FOUND,
                "the requested resource could not be found",
            )
            .into_response(),
            Self::NoAuthorizationHeader => {
                error_response(StatusCode::UNAUTHORIZED, "no authorization header").into_response()
            }
            Self::ExpiredToken => {
                error_response(StatusCode::UNAUTHORIZED, "your token has expired").into_response()
            }
            Self::DuplicatedUsername(username) => error_response(
                StatusCode::CONFLICT,
                format!("username {} already exists", username).as_str(),
            )
            .into_response(),
            Self::InvalidTimeString(err) => {
                info!("{:?}", err);
                error_response(StatusCode::BAD_REQUEST, "invalid time string").into_response()
            }
            Self::InternalServerError(s) => {
                error!("{}", s);
                error_response(StatusCode::INTERNAL_SERVER_ERROR, s).into_response()
            }
        }
    }
}
