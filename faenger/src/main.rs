use crate::models::NewUser;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHasher};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use diesel::SqliteConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use serde::Deserialize;
use std::env;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
pub mod models;
pub mod schema;

fn setup_database() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url).expect("Could not establish sqlite connection")
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<SqliteConnection>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let state = AppState {
        db: Arc::new(Mutex::new(setup_database())),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/register", post(register))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4567")
        .await
        .expect("Could not bind tcp listener");

    axum::serve(listener, app)
        .await
        .expect("Axum stopped serving 😤")
}
async fn root() -> &'static str {
    "🦕"
}

#[derive(Deserialize)]
struct UserAuthReq {
    username: String,
    password: String,
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<UserAuthReq>,
) -> (StatusCode, String) {
    use crate::schema::users;

    // surely the defaults will be sane
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let pw_hash = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "password issue".to_string(),
            );
        }
    };

    let new_user = NewUser::new(payload.username, pw_hash.to_string());

    let db_response = {
        let mut db = state.db.lock().expect("mutex was poisoned :(");
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(db.deref_mut())
    };

    match db_response {
        Ok(_) => {}
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "not ok".to_string()),
    }

    (StatusCode::CREATED, "ok".to_string())
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<UserAuthReq>,
) -> (StatusCode, String) {
    use crate::schema::users;
    (StatusCode::CREATED, "ok".to_string())
}
