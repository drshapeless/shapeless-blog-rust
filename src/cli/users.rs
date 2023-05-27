use std::{eprintln, println};

use sqlx::PgPool;

use crate::data::users;

pub async fn create_user(username: String, password: String, pool: PgPool) {
    let mut conn = pool.acquire().await.unwrap();

    match users::get_user_by_username(username.clone(), &mut conn).await {
        Ok(user) => {
            eprintln!("User {} already exists", user.username);
        }
        Err(err) => match err {
            sqlx::Error::RowNotFound => {
                let new_user = users::NewUser { username, password };
                let user = users::create_user(new_user, &mut conn).await.unwrap();

                println!(
                    "User {} created
id: {}",
                    user.username, user.id
                );
            }
            _ => {
                eprintln!("{}", err);
            }
        },
    }
}
