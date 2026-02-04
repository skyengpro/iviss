use axum::{response::Json, extract::State};
use serde_json::{json, Value};
use crate::db::DbPools;

pub async fn login(State(_pools): State<DbPools>) -> Json<Value> {
    Json(json!({"token": "placeholder"}))
}
