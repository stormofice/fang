use super::extractors::AuthInfo;
use crate::AppState;
use crate::users::models::{NewUser, User};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use diesel::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserAuthReq {
    pub username: String,
    pub password: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<UserAuthReq>,
) -> (StatusCode, String) {
    {
        let cfg = state.config.read().unwrap();
        if !cfg.allow_signups {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Sorry, signups are currently disabled".to_string(),
            );
        }
    }

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
        let mut db = match state.db.get() {
            Ok(conn) => conn,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database connection timeout".to_string(),
                );
            }
        };
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut db)
    };

    match db_response {
        Ok(_) => {}
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "not ok".to_string()),
    }

    (StatusCode::CREATED, "ok".to_string())
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<UserAuthReq>,
) -> (StatusCode, String) {
    use crate::schema::users::dsl::*;

    let user_rows = {
        let mut db = match state.db.get() {
            Ok(conn) => conn,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database connection timeout".to_string(),
                );
            }
        };
        match users
            .filter(name.eq(&payload.username))
            .select(User::as_select())
            .load(&mut db)
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

pub async fn check(_auth: AuthInfo) -> (StatusCode, String) {
    (StatusCode::OK, "you good".to_string())
}
