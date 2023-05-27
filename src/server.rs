use std::net::SocketAddr;

use crate::app::{self, AppState};

pub async fn serve(state: AppState) {
    let app = app::create_app(state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], state.config.socket));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
