use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::mpsc;
use tokio::sync::mpsc as tokio_mpsc; // Use tokio's MPSC for commands
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower_http::cors::{Any, CorsLayer};

use crate::engine::{EngineCommand, EngineRequest};
use crate::store::Scene;
use crate::types::Virtual;

// Command enum for the API manager thread
pub enum ApiCommand {
    Restart { port: u16 },
}

#[derive(Clone)]
struct ApiState {
    engine_command_tx: mpsc::Sender<EngineCommand>,
    engine_state_tx: mpsc::Sender<EngineRequest>,
}

// The main function for the new management thread, now fully async
pub async fn api_server_manager(
    mut api_command_rx: tokio_mpsc::Receiver<ApiCommand>,
    engine_command_tx: mpsc::Sender<EngineCommand>,
    engine_state_tx: mpsc::Sender<EngineRequest>,
    initial_port: u16,
) {
    let mut server_handle: Option<JoinHandle<()>> = None;

    // Helper closure to spawn the server task
    let start_server_task = |port: u16| {
        let state = ApiState {
            engine_command_tx: engine_command_tx.clone(),
            engine_state_tx: engine_state_tx.clone(),
        };

        tokio::spawn(async move {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any);

            let app = Router::new()
                .route("/scenes", get(get_scenes_handler))
                .route(
                    "/scenes/:id/activate",
                    post(activate_scene_handler).get(activate_scene_handler),
                )
                .route("/virtuals", get(get_virtuals_handler))
                .route("/virtuals/:id/effects/stop", post(stop_effect_handler))
                .with_state(state)
                .layer(cors);

            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            println!("[API] HTTP server starting on {}", addr);

            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => listener,
                Err(e) => {
                    eprintln!("[API] ERROR: Failed to bind to port {}: {}", port, e);
                    return;
                }
            };

            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("[API] Server error: {}", e);
            }
        })
    };

    // Start the server initially
    server_handle = Some(start_server_task(initial_port));

    // Async loop to listen for commands
    while let Some(command) = api_command_rx.recv().await {
        match command {
            ApiCommand::Restart { port } => {
                println!("[API MANAGER] Received restart command for port {}", port);
                if let Some(handle) = server_handle.take() {
                    println!("[API MANAGER] Aborting old server task...");
                    handle.abort(); // Shut down the old server task
                }
                server_handle = Some(start_server_task(port));
            }
        }
    }
}

async fn request_engine_state<T: Send + 'static>(
    tx: &mpsc::Sender<EngineRequest>,
    request_builder: impl FnOnce(mpsc::Sender<T>) -> EngineRequest + Send + 'static,
) -> Result<Json<T>, StatusCode> {
    let (responder_tx, responder_rx) = oneshot::channel();
    let engine_state_tx = tx.clone();

    tokio::task::spawn_blocking(move || {
        let (sync_responder_tx, sync_responder_rx) = mpsc::channel();
        if engine_state_tx
            .send(request_builder(sync_responder_tx))
            .is_ok()
        {
            if let Ok(data) = sync_responder_rx.recv() {
                let _ = responder_tx.send(data);
            }
        }
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match responder_rx.await {
        Ok(data) => Ok(Json(data)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_scenes_handler(State(state): State<ApiState>) -> Result<Json<Vec<Scene>>, StatusCode> {
    request_engine_state(&state.engine_state_tx, |tx| EngineRequest::GetScenes(tx)).await
}
async fn activate_scene_handler(
    State(state): State<ApiState>,
    Path(scene_id): Path<String>,
) -> StatusCode {
    let command = EngineCommand::ActivateScene(scene_id);
    match state.engine_command_tx.send(command) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
async fn get_virtuals_handler(
    State(state): State<ApiState>,
) -> Result<Json<Vec<Virtual>>, StatusCode> {
    request_engine_state(&state.engine_state_tx, |tx| EngineRequest::GetVirtuals(tx)).await
}
async fn stop_effect_handler(
    State(state): State<ApiState>,
    Path(virtual_id): Path<String>,
) -> StatusCode {
    let command = EngineCommand::StopEffect { virtual_id };
    match state.engine_command_tx.send(command) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
