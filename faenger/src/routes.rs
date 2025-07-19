use crate::auth::handlers::{check, login, register};
use crate::links::handlers::{list, save};
use crate::AppState;
use axum::routing::{get, post};
use axum::Router;

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
}