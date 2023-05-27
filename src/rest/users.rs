use axum::extract::{Path, State};

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{middleware, Extension, Json, Router};
use serde::Serialize;

use crate::data::tokens::{self, Token};
use crate::data::users::{self, hash_password};

use crate::app::AppState;

use super::errors::ApiError;
use super::helpers::get_conn_from_pool;
use super::middlewares::auth;
use super::Result;

pub fn routes(state: AppState) -> Router {
    let protected = Router::new()
        .route("/user/", post(create_user_handler))
        .route(
            "/user/:id",
            get(show_user_handler).put(update_user).delete(delete_user),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state.clone());

    let public = Router::new()
        .route("/authentication", post(create_user_token_handler))
        .with_state(state);

    Router::new().merge(protected).merge(public)
}

async fn create_user_handler(
    State(state): State<AppState>,
    Json(new_user): Json<users::NewUser>,
) -> Result<(StatusCode, Json<User>)> {
    let mut conn = get_conn_from_pool(state.db).await?;

    // This mess is to prevent user id from bumping up in fail insertion.
    match users::get_user_by_username(new_user.username.clone(), &mut conn).await {
        Ok(user) => Err(ApiError::DuplicatedUsername(user.username)),
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                let user = users::create_user(new_user, &mut conn)
                    .await
                    .map_err(ApiError::SqlxError)?;

                let user = User::from_data_user(user);

                Ok((StatusCode::CREATED, Json(user)))
            }
            _ => Err(ApiError::SqlxError(err)),
        },
    }
}

#[derive(Serialize)]
struct User {
    id: i64,
    username: String,
}

impl User {
    fn from_data_user(user: users::User) -> Self {
        User {
            id: user.id,
            username: user.username,
        }
    }
}

async fn show_user_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<User>> {
    let mut conn = get_conn_from_pool(state.db).await?;

    let user = users::get_user(id, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    Ok(Json(User::from_data_user(user)))
}

async fn update_user(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(id): Path<i64>,
    Json(updated_user): Json<users::NewUser>,
) -> Result<StatusCode> {
    if id != user_id {
        return Err(ApiError::Unauthorized);
    }

    let mut conn = get_conn_from_pool(state.db).await?;

    let mut old_user = users::get_user(id, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    old_user.username = updated_user.username;
    old_user.hashed_password = hash_password(updated_user.password);

    let _user = users::update_user(old_user, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_user(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    if id != user_id {
        return Err(ApiError::Unauthorized);
    }

    let mut conn = get_conn_from_pool(state.db).await?;

    let success = users::delete_user(id, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    if !success {
        return Err(ApiError::Unauthorized);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn create_user_token_handler(
    State(state): State<AppState>,
    Json(user): Json<users::NewUser>,
) -> Result<(StatusCode, Json<Token>)> {
    let mut conn = get_conn_from_pool(state.db).await?;

    let db_user = users::get_user_by_username(user.username, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    let verified =
        bcrypt::verify(user.password, &db_user.hashed_password).map_err(ApiError::BcryptError)?;

    if verified {
        let token = tokens::insert_token_for_user(
            db_user.id,
            chrono::Utc::now()
                .checked_add_signed(chrono::Duration::days(1))
                .unwrap(),
            &mut conn,
        )
        .await
        .map_err(ApiError::SqlxError)?;

        Ok((StatusCode::CREATED, Json(token)))
    } else {
        Err(ApiError::Unauthorized)
    }
}
