use crate::auth::extractors::AuthInfo;
use crate::links::models::{Fang, NewFang};
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;
use serde::Deserialize;
use std::ops::DerefMut;

pub async fn list(State(state): State<AppState>, auth_info: AuthInfo) -> (StatusCode, Json<Vec<Fang>>) {
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