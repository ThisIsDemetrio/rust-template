use std::time::Duration;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use mongodb::{Database, bson::doc};
use serde_json::{Value, json};

///////////
// Helpers
///////////

///////////
// APIs
///////////
async fn livez() -> Json<Value> {
    Json(json!({ "status": "OK"}))
}

async fn readyz(
    State(database): State<Database>
) -> (StatusCode, Json<Value>) {
    match tokio::time::timeout(
        Duration::from_secs(2),
        database.run_command(doc! { "ping": 1 }),
    )
    .await
    {
        Ok(Ok(_)) => (StatusCode::OK, Json(json!({ "status": "OK"}))),
        Ok(Err(e)) => {
            tracing::warn!("Readiness probe failed with error: {e}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "KO"})),
            )
        }
        Err(_) => {
            tracing::warn!("Readiness probe failed after default timeout");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "status": "KO"})),
            )
        }
    }
}

///////////
// Router
///////////

pub fn router(database: Database) -> Router {
    Router::new()
        .route("/livez", get(livez))
        .route("/readyz", get(readyz))
        .route("/healthz", get(livez))
        .with_state(database)
}
