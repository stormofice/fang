use crate::AppState;
use crate::auth::handlers::{check, login, register};
use crate::links::handlers::{forget, has, list, save};
use axum::Router;
use axum::routing::{delete, get, post};

async fn root() -> &'static str {
    "🦕"
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(root))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/check", get(check))
        .route("/faenge/list", get(list))
        .route("/faenge/save", post(save))
        .route("/faenge/forget", delete(forget))
        .route("/faenge/has", get(has))
}
