pub mod server;
mod database;
mod data_service;

use std::error::Error;
use std::sync::Arc;
use axum::routing::get;
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

use crate::server::WarhorseServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let (layer, io) = SocketIo::new_layer();
    let horse_server = Arc::new(Mutex::new(crate::WarhorseServer::<crate::database::db_in_memory::InMemoryDatabase>::new(io)));

    let horse_server_clone = horse_server.clone();
    horse_server_clone.lock().await.io().ns("/", move |socket: SocketRef, Data::<Value>(data)| {
        async move {
            server::handle_connection(socket, data, horse_server_clone.clone()).await;
        }
    });

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer);

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}