use crate::data::tokens;
use axum::extract::State;

use crate::app::AppState;

use super::{errors::ApiError, helpers::get_conn_from_pool, Result};

pub async fn auth<B>(
    State(state): State<AppState>,
    mut req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> Result<impl axum::response::IntoResponse> {
    let head = req.headers().get(axum::http::header::AUTHORIZATION);

    let header = head.ok_or(ApiError::NoAuthorizationHeader)?;

    let (name, contents) = header
        .to_str()
        .map_err(|_| ApiError::NoAuthorizationHeader)?
        .split_once(' ')
        .ok_or(ApiError::NoAuthorizationHeader)?;

    if name != "Bearer" {
        return Err(ApiError::Unauthorized);
    }

    let mut conn = get_conn_from_pool(state.db).await?;

    let token = tokens::get_token_by_token_string(contents, &mut conn)
        .await
        .map_err(ApiError::SqlxError)?;

    if token.is_expired() {
        // When detecting any expired token, delete all the expired
        // token in the database.

        // Success or not, doesn't matter.
        tokens::delete_all_expired_tokens(&mut conn)
            .await
            .map_err(ApiError::SqlxError)?;

        return Err(ApiError::ExpiredToken);
    }

    req.extensions_mut().insert(token.user_id);

    Ok(next.run(req).await)
}
