use std::{error::Error, sync::mpsc};

use axum::{Router, routing};
use socketioxide::{
  SocketIo,
  extract::{Data, SocketRef},
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::{tui::tuiapp::UiEvent, utils::apps::AppDetail};

pub async fn on_connect(socket: SocketRef, ui_tx: mpsc::Sender<UiEvent>) {
  info!(ns = socket.ns(), ?socket.id, "Socket.IO connected");
  ui_tx.send(UiEvent::Connected).ok();

  let ui_tx_for_apps = ui_tx.clone();
  socket.on(
    "loadApps",
    move |socket: SocketRef, Data::<Vec<AppDetail>>(data)| {
      let ui_tx = ui_tx_for_apps.clone();
      async move {
        info!(?data, "Received message from client:");
        ui_tx.send(UiEvent::AppsReceived(data.clone())).ok();
        socket
          .emit("feedback", &format!("Received {} apps.", data.len()))
          .ok();
      }
    },
  );
}

pub async fn start_socket_server(
  ui_tx: mpsc::Sender<UiEvent>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
  // let _ = tracing_subscriber::fmt::try_init();
  let (layer, io) = SocketIo::new_layer();

  let ui_tx_for_connect = ui_tx.clone();
  io.ns("/", move |socket| {
    let ui_tx = ui_tx_for_connect.clone();
    async move {
      on_connect(socket, ui_tx).await;
    }
  });

  let app = Router::new()
    .route(
      "/",
      routing::get(|| async {
        "Well, this server isn't meant for direct HTTP requests, but I appreciate your interest!"
      }),
    )
    .layer(
      ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(layer),
    );

  // info!("Starting socket.io server...");

  let listener = TcpListener::bind("0.0.0.0:7212").await?;
  axum::serve(listener, app).await?;

  Ok(())
}
