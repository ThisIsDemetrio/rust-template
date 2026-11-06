use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::error::Error;
use serde_json::json;

pub enum ApiError {
    BadRequest(String),
    NotFound,
    Internal(anyhow::Error),
}

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        ApiError::Internal(e.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::BadRequest(e) => (StatusCode::BAD_REQUEST, e),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "notfound".to_string()),
            ApiError::Internal(error) => {
                tracing::error!("{error:#}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
            }
        };
        (status, Json(json!({"error": msg}))).into_response()
    }
}
