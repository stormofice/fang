use crate::AppState;
use crate::auth::extractors::AuthInfo;
use crate::links::models::{Fang, NewFang};
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use diesel::prelude::*;
use serde::Deserialize;
use std::ops::DerefMut;

pub async fn list(
    State(state): State<AppState>,
    auth_info: AuthInfo,
) -> (StatusCode, Json<Vec<Fang>>) {
    let mut db = match state.db.get() {
        Ok(conn) => conn,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    };
    match Fang::belonging_to(&auth_info.0)
        .select(Fang::as_select())
        .load(&mut db)
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

#[derive(Debug, Deserialize)]
pub struct HasFangReq {
    pub url: String,
}
pub async fn has(
    State(state): State<AppState>,
    auth_info: AuthInfo,
    Query(payload): Query<HasFangReq>,
) -> StatusCode {
    use crate::schema::faenge::dsl::*;

    log::debug!("Received has request: {:?}", payload);

    let mut db = match state.db.get() {
        Ok(conn) => conn,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    match Fang::belonging_to(&auth_info.0)
        .filter(url.eq(&payload.url))
        .filter(user_id.eq(&auth_info.0.id))
        .count()
        .get_result(db.deref_mut())
    {
        Ok(res) => match res {
            0 => StatusCode::NOT_FOUND,
            1 => StatusCode::FOUND,
            _ => {
                log::error!(
                    "URL {:?} saved more than once by {:?}",
                    &payload,
                    &auth_info.0
                );
                StatusCode::INTERNAL_SERVER_ERROR
            }
        },
        Err(e) => {
            log::error!(
                "Error while listing faenge for: {:?}, error: {:?}",
                &auth_info.0,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SaveFangReq {
    pub url: String,
    pub title: Option<String>,
}

pub async fn save(
    State(state): State<AppState>,
    auth_info: AuthInfo,
    Json(payload): Json<SaveFangReq>,
) -> (StatusCode, String) {
    use crate::schema::faenge;

    log::debug!("Received save request: {:?}", payload);

    let new_fang = NewFang::new(payload.url, payload.title, auth_info.0.id);
    let mut db = match state.db.get() {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection timeout".to_string(),
            );
        }
    };
    match diesel::insert_into(faenge::table)
        .values(&new_fang)
        .execute(&mut db)
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
