use axum::{
    routing::{get, post},
    Router
};

use std::io;
use std::net::SocketAddr;
use tracing::Level;

mod handlers;
mod models;
mod db_connection;
mod config;
mod services;
mod utils;
mod redis_instance;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_writer(io::stdout)
        .init();

    // Initialize the router
    let app = Router::new()
    .route("/hb", get(|| async { "OK" }))
    .route("/register", post(handlers::user_handler::register))
    .route("/login", post(handlers::user_handler::login))
    .route("/user_info", get(handlers::user_handler::user_info))
    .route("/logout",post(handlers::user_handler::logout))
    .route("/update_info", post(handlers::user_handler::update_user_info))
    .route("/delete_user", post(handlers::user_handler::delete_user));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
    
}