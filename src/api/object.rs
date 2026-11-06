use crate::api::error::ApiError;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use futures::TryStreamExt;
use mongodb::{
    Collection,
    bson::{self, Document, doc, oid::ObjectId},
    options::FindOptions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

///////////
// Structs
///////////

#[derive(Deserialize)]
struct Page {
    limit: Option<i64>,
    skip: Option<u64>,
}

#[derive(Deserialize, Serialize)]
struct ObjectDocument {
    #[serde(serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string")]
    _id: ObjectId,
    value: serde_json::Value,
}

#[derive(Serialize)]
struct ObjectCreateResponse {
    #[serde(serialize_with = "bson::serde_helpers::serialize_object_id_as_hex_string")]
    _id: ObjectId,
}

///////////
// Helpers
///////////

pub fn parse_id(id: &str) -> Result<ObjectId, ApiError> {
    ObjectId::parse_str(id).map_err(|_| ApiError::BadRequest("invalid id".into()))
}

///////////
// APIs
///////////
async fn list(
    State(collection): State<Collection<Document>>,
    Query(page): Query<Page>,
) -> Result<Json<Vec<ObjectDocument>>, ApiError> {
    let opts = FindOptions::builder()
        .sort(doc! { "_id": 1 })
        .limit(page.limit.unwrap_or(20).clamp(1, 100))
        .skip(page.skip.unwrap_or(0))
        .build();

    let docs: Vec<ObjectDocument> = collection
        .clone_with_type::<ObjectDocument>()
        .find(doc! {})
        .with_options(opts)
        .await?
        .try_collect()
        .await?;

    Ok(Json(docs))
}

async fn create(
    State(collection): State<Collection<Document>>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<ObjectCreateResponse>), ApiError> {
    let doc = bson::to_bson(&body)
        .map_err(|_| ApiError::BadRequest("Impossible to save value as BSON".into()))?;

    let id = collection
        .insert_one(doc! {"value": &doc})
        .await?
        .inserted_id
        .as_object_id()
        .ok_or_else(|| {
            ApiError::Internal(anyhow::anyhow!("Internal Error generating MongoDB _id"))
        })?;

    Ok((StatusCode::CREATED, Json(ObjectCreateResponse { _id: id })))
}

async fn get_one(
    State(collection): State<Collection<Document>>,
    Path(id): Path<String>,
) -> Result<Json<ObjectDocument>, ApiError> {
    let document: ObjectDocument = collection
        .clone_with_type::<ObjectDocument>()
        .find_one(doc! { "_id": parse_id(&id)? })
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(document))
}

async fn delete_one(
    State(collection): State<Collection<Document>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    match collection
        .delete_one(doc! { "_id": parse_id(&id)? })
        .await?
        .deleted_count
    {
        0 => Err(ApiError::NotFound),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}

///////////
// Router
///////////


pub fn router(collection: Collection<Document>) -> Router {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).delete(delete_one))
        .with_state(collection)
}
