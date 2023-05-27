use app::AppState;
use clap::Parser;
use cli::users::create_user;
use config::Config;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tracing::error;
use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    prelude::__tracing_subscriber_SubscriberExt,
    Registry,
};

mod app;
mod cli;
mod config;
mod data;
// mod log;
mod rest;
mod server;
mod web;

#[tokio::main]
async fn main() {
    let config = Config::parse();

    let file_appender =
        tracing_appender::rolling::daily(config.log_directory.clone(), "shapeless-blog.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = Registry::default()
        .with(
            fmt::Layer::default()
                .with_writer(file_writer.with_max_level(tracing::Level::WARN))
                .with_ansi(false),
        )
        .with(
            fmt::Layer::default().with_writer(std::io::stdout.with_max_level(tracing::Level::INFO)),
        );

    tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");

    let options = PgConnectOptions::new()
        .host(config.host.as_str())
        .username(config.username.as_str())
        .database(config.database.as_str());

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect_with(options)
        .await
        .unwrap();

    if config.migrate {
        match sqlx::migrate!().run(&db).await {
            Ok(_) => return,
            Err(err) => {
                error!("{:?}", err);
                return;
            }
        }
    }

    if config.create_user {
        if config.new_username.is_none() {
            error!("username is empty");
            return;
        }

        if config.new_password.is_none() {
            error!("password is empty");
            return;
        }

        let username = config.new_username.unwrap();
        let password = config.new_password.unwrap();

        create_user(username, password, db).await;
        return;
    }

    let state = AppState::new(db, config);

    server::serve(state).await;
}
