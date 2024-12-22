pub mod server;
mod database;
mod data_service;
mod utils;
mod error;
mod i18n;
mod config;

use std::sync::Arc;
use axum::routing::get;
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use crate::error::ServerError;
use crate::server::WarhorseServer;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())
        .map_err(|e| ServerError(e.to_string()))?;

    let (layer, io) = SocketIo::new_layer();
    let server = Arc::new(Mutex::new(
        WarhorseServer::<database::db_in_memory::InMemoryDatabase>::new(io, "")
    ));

    let server_clone = server.clone();
    {
        let server = server_clone.clone();
        let io = server.lock().await.io().clone();
        io.ns("/", move |socket: SocketRef, Data::<Value>(data)| {
            let server = server.clone();
            async move {
                server::handle_connection(socket, data, server).await;
            }
        });
    }

    let app = axum::Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .layer(layer);

    info!("Starting server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await
        .map_err(|e| ServerError(e.to_string()))?;

    axum::serve(listener, app).await
        .map_err(|e| ServerError(e.to_string()))?;

    Ok(())
}