use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::mpsc::{self, Receiver};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower_http::cors::{Any, CorsLayer};

use crate::engine::{EngineCommand, EngineRequest};
use crate::store::{EngineState, Scene}; // <-- Import EngineState
use crate::types::{Device, Virtual}; // <-- Import Device

pub enum ApiCommand {
    Restart { port: u16 },
}

#[derive(Clone)]
struct ApiState {
    engine_command_tx: mpsc::Sender<EngineCommand>,
    engine_state_tx: mpsc::Sender<EngineRequest>,
}

pub async fn api_server_manager(
    api_command_rx: Receiver<ApiCommand>,
    engine_command_tx: mpsc::Sender<EngineCommand>,
    engine_state_tx: mpsc::Sender<EngineRequest>,
    initial_port: u16,
) {
    #[allow(unused)]
    let mut server_handle: Option<JoinHandle<()>> = None;

    let start_server_task = move |port: u16| {
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
                // --- START: ADD NEW ROUTES ---
                .route("/state", get(get_full_state_handler))
                .route("/devices", get(get_devices_handler))
                // --- END: ADD NEW ROUTES ---
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

    server_handle = Some(start_server_task(initial_port));

    tokio::task::spawn_blocking(move || {
        for command in api_command_rx {
            match command {
                ApiCommand::Restart { port } => {
                    println!("[API MANAGER] Received restart command for port {}", port);
                    if let Some(handle) = server_handle.take() {
                        println!("[API MANAGER] Aborting old server task...");
                        handle.abort();
                    }
                    server_handle = Some(start_server_task(port));
                }
            }
        }
    });

    let (_tx, rx) = tokio::sync::oneshot::channel::<()>();
    let _ = rx.await;
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

// --- START: NEW HANDLERS ---
// This handler requires a new EngineRequest variant.
async fn get_full_state_handler(
    State(state): State<ApiState>,
) -> Result<Json<EngineState>, StatusCode> {
    request_engine_state(&state.engine_state_tx, |tx| EngineRequest::GetFullState(tx)).await
}

async fn get_devices_handler(
    State(state): State<ApiState>,
) -> Result<Json<Vec<Device>>, StatusCode> {
    request_engine_state(&state.engine_state_tx, |tx| EngineRequest::GetDevices(tx)).await
}
// --- END: NEW HANDLERS ---

async fn get_scenes_handler(State(state): State<ApiState>) -> Result<Json<Vec<Scene>>, StatusCode> {
    request_engine_state(&state.engine_state_tx, |tx| EngineRequest::GetScenes(tx)).await
}
async fn activate_scene_handler(
    State(state): State<ApiState>,
    Path(scene_id): Path<String>,
) -> StatusCode {
    let command = EngineCommand::ActivateScene(scene_id);
    if state.engine_command_tx.send(command).is_ok() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
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
    if state.engine_command_tx.send(command).is_ok() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
