use clap::Parser;

#[derive(Parser)]
pub struct Config {
    #[arg(long, default_value = "localhost")]
    pub host: String,

    #[arg(long, default_value = "jacky")]
    pub username: String,

    #[arg(long, default_value = "shapeless-blog")]
    pub database: String,

    #[arg(long, default_value_t = 9398)]
    pub socket: u16,

    #[arg(long, default_value = "log/")]
    pub log_directory: String,

    #[arg(long)]
    pub create_user: bool,

    #[arg(long)]
    pub new_username: Option<String>,

    #[arg(long)]
    pub new_password: Option<String>,

    #[arg(long)]
    pub migrate: bool,
}
