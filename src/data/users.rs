use serde::{Deserialize, Serialize};
use sqlx::PgConnection;

use super::Result;

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub hashed_password: String,
    version: i64,
}

#[derive(Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

pub fn hash_password(password: String) -> String {
    bcrypt::hash(password.as_str(), bcrypt::DEFAULT_COST).expect("Cannot hash password")
}

pub async fn create_user(new_user: NewUser, conn: &mut PgConnection) -> Result<User> {
    let query = "
INSERT INTO users (username, hashed_password)
VALUES ($1, $2)
RETURNING *";

    let hashed_password = hash_password(new_user.password);

    let user = sqlx::query_as::<_, User>(query)
        .bind(new_user.username)
        .bind(hashed_password)
        .fetch_one(conn)
        .await?;

    Ok(user)
}

pub async fn get_user(id: i64, conn: &mut PgConnection) -> Result<User> {
    let q = "
SELECT * FROM users
WHERE id = $1 ";

    let user = sqlx::query_as::<_, User>(q)
        .bind(id)
        .fetch_one(conn)
        .await?;

    Ok(user)
}

pub async fn get_user_by_username(username: String, conn: &mut PgConnection) -> Result<User> {
    let q = "
SELECT * FROM users
WHERE username = $1 ";

    let user = sqlx::query_as::<_, User>(q)
        .bind(username)
        .fetch_one(conn)
        .await?;

    Ok(user)
}

pub async fn update_user(updated_user: User, conn: &mut PgConnection) -> Result<User> {
    let q = "
UPDATE users
SET username = $1, hashed_password = $2, version = version + 1
WHERE id = $3 AND version = $4
RETURNING *";

    let user = sqlx::query_as::<_, User>(q)
        .bind(updated_user.username)
        .bind(updated_user.hashed_password)
        .bind(updated_user.id)
        .bind(updated_user.version)
        .fetch_one(conn)
        .await?;

    Ok(user)
}

pub async fn delete_user(id: i64, conn: &mut PgConnection) -> Result<bool> {
    let q = "
DELETE FROM users
WHERE id = $1";

    let result = sqlx::query(q).bind(id).execute(conn).await?;

    Ok(result.rows_affected() > 0)
}
