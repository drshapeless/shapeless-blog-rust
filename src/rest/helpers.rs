use super::errors::ApiError;
use super::Result;
use sqlx::{pool::PoolConnection, PgPool, Postgres, Transaction};

pub async fn get_conn_from_pool(pool: PgPool) -> Result<PoolConnection<Postgres>> {
    let conn = pool.acquire().await.map_err(ApiError::SqlxError)?;
    Ok(conn)
}

pub async fn get_tx_from_pool(pool: PgPool) -> Result<Transaction<'static, Postgres>> {
    let tx = pool.begin().await.map_err(ApiError::SqlxError)?;

    Ok(tx)
}
