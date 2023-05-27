use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::Serialize;
use sqlx::{FromRow, PgConnection};

use super::Result;

#[derive(FromRow, Serialize)]
pub struct Token {
    pub user_id: i64,
    token: String,
    expired_time: DateTime<Utc>,
}

impl Token {
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        now >= self.expired_time
    }
}

fn generate_token() -> String {
    let mut token = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token);

    hex::encode(token)
}

pub async fn get_token_by_token_string(
    token_string: &str,
    conn: &mut PgConnection,
) -> Result<Token> {
    let token = sqlx::query_as::<_, Token>(
        "SELECT user_id, token, expired_time
FROM tokens
WHERE token = $1",
    )
    .bind(token_string)
    .fetch_one(conn)
    .await?;

    Ok(token)
}

pub async fn insert_token_for_user(
    user_id: i64,
    expired_time: DateTime<Utc>,
    conn: &mut PgConnection,
) -> Result<Token> {
    let token_string = generate_token();

    let token = sqlx::query_as::<_, Token>(
        "INSERT INTO tokens (user_id, token, expired_time)
VALUES ($1, $2, $3)
RETURNING *",
    )
    .bind(user_id)
    .bind(token_string)
    .bind(expired_time)
    .fetch_one(conn)
    .await?;

    Ok(token)
}

pub async fn _delete_all_tokens_for_user(user_id: i64, conn: &mut PgConnection) -> Result<bool> {
    let result = sqlx::query(
        "DELETE FROM tokens
WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(conn)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_all_expired_tokens(conn: &mut PgConnection) -> Result<bool> {
    let result = sqlx::query(
        "DELETE FROM tokens
WHERE NOW() > expired_time",
    )
    .execute(conn)
    .await?;

    Ok(result.rows_affected() > 0)
}
