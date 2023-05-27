pub mod blogs;
pub mod tags;
pub mod tokens;
pub mod users;

pub type Result<T, E = sqlx::Error> = std::result::Result<T, E>;
