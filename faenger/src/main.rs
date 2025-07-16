use crate::models::{NewUser, User};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use diesel::SqliteConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use serde::Deserialize;
use std::convert::Infallible;
use std::env;
use std::net::ToSocketAddrs;
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
    pretty_env_logger::init_timed();
    dotenv().ok();

    let state = AppState {
        db: Arc::new(Mutex::new(setup_database())),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/register", post(register))
        .route("/login", post(login))
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
    use crate::schema::users::dsl::*;

    let user_rows = {
        let mut db = state.db.lock().expect("mutex was poisoned :(");
        match users
            .filter(name.eq(&payload.username))
            .select(User::as_select())
            .load(db.deref_mut())
        {
            Ok(rows) => rows,
            Err(e) => {
                log::error!("Could not retrieve user info from db: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "sowwy".to_string());
            }
        }
    };

    if user_rows.is_empty() {
        return (StatusCode::NOT_FOUND, "where are you".to_string());
    } else if user_rows.len() != 1 {
        log::error!(
            "Multiple users found for name: {:?} - Users: {:?}",
            &payload.username,
            user_rows
        );
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "we got some internalized issues".to_string(),
        );
    }

    let db_user = user_rows.first().expect("this cannot happen");
    assert_eq!(db_user.name, payload.username, "database filtering defect");

    let argon2 = Argon2::default();
    let saved_pw_hash = match PasswordHash::new(db_user.password_hash.as_str()) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!(
                "Invalid password hash in db: {:?} for user: {:?}",
                e,
                db_user
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "we got some internalized issues".to_string(),
            );
        }
    };

    match argon2.verify_password(payload.password.as_bytes(), &saved_pw_hash) {
        Ok(_) => {}
        Err(e) => {
            log::warn!("Failed login attempt for user {}: {:?}", db_user.name, e);
            return (StatusCode::FORBIDDEN, "nuh uh, wrong password".to_string());
        }
    }

    (StatusCode::OK, db_user.api_key.clone())
}
