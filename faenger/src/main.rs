#![allow(clippy::uninlined_format_args)]
use crate::models::{Fang, NewFang, NewUser, User};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
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
    pretty_env_logger::init_timed();
    dotenv().ok();

    let state = AppState {
        db: Arc::new(Mutex::new(setup_database())),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/check", get(check))
        .route("/faenge/list", get(list))
        .route("/faenge/save", post(save))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4567")
        .await
        .expect("Could not bind tcp listener");

    axum::serve(listener, app)
        .await
        .expect("Axum stopped serving 😤")
}

pub struct AuthInfo(User);

impl<S> FromRequestParts<S> for AuthInfo
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let req_api_key = parts
            .headers
            .get("X-Api-Key")
            .and_then(|hv| hv.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing X-Api-Key header".to_string(),
            ))?;

        if req_api_key.len() != 32 {
            return Err((StatusCode::BAD_REQUEST, "Malformed X-Api-Key".to_string()));
        }

        let app_state = AppState::from_ref(state);

        let users = {
            let mut db = app_state.db.lock().expect("Mutex was poisoned :(");
            use crate::schema::users::dsl::*;

            match users
                .filter(api_key.eq(req_api_key))
                .select(User::as_select())
                .load(db.deref_mut())
            {
                Ok(rows) => rows,
                Err(e) => {
                    log::error!("Could not access database for api key auth: {:?}", e);
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, "uhm".to_string()));
                }
            }
        };

        if users.is_empty() {
            Err((
                StatusCode::UNAUTHORIZED,
                "Invalid API key, nuh uh".to_string(),
            ))
        } else if users.len() == 1 {
            Ok(AuthInfo(users.first().unwrap().clone()))
        } else {
            log::error!(
                "Found shared API Keys, Key: {}, Count: {}, Users: {:?}",
                req_api_key,
                users.len(),
                users
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Sorry, we messed up our keys".to_string(),
            ))
        }
    }
}

async fn root() -> &'static str {
    "🦕"
}

async fn check(_auth: AuthInfo) -> (StatusCode, String) {
    (StatusCode::OK, "you good".to_string())
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

async fn list(State(state): State<AppState>, auth_info: AuthInfo) -> (StatusCode, Json<Vec<Fang>>) {
    let mut db = state.db.lock().expect("Mutex was poisoned :(");
    match Fang::belonging_to(&auth_info.0)
        .select(Fang::as_select())
        .load(db.deref_mut())
    {
        Ok(res) => (StatusCode::OK, Json(res)),
        Err(e) => {
            log::error!(
                "Error while listing faenge for: {:?}, error: {:?}",
                &auth_info.0,
                e
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

#[derive(Deserialize)]
struct SaveFangReq {
    url: String,
    title: Option<String>,
}

async fn save(
    State(state): State<AppState>,
    auth_info: AuthInfo,
    Json(payload): Json<SaveFangReq>,
) -> (StatusCode, String) {
    use crate::schema::faenge;

    let new_fang = NewFang::new(payload.url, payload.title, auth_info.0.id);
    let mut db = state.db.lock().expect("Mutex was poisoned :(");
    match diesel::insert_into(faenge::table)
        .values(&new_fang)
        .execute(db.deref_mut())
    {
        Ok(_) => (StatusCode::OK, "caught it".to_string()),
        Err(e) => {
            log::error!(
                "Could not insert new fang: {:?} for user: {:?} due to: {:?}",
                new_fang,
                auth_info.0,
                e
            );
            (StatusCode::INTERNAL_SERVER_ERROR, "troubles".to_string())
        }
    }
}
