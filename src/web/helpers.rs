use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};

use super::errors::WebError;
use super::Result;

pub async fn get_conn_from_pool(pool: PgPool) -> Result<PoolConnection<Postgres>> {
    let conn = pool.acquire().await.map_err(WebError::SqlxError)?;
    Ok(conn)
}
