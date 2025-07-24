use crate::AppState;
use crate::auth::extractors::AuthInfo;
use crate::links::models::{Fang, NewFang};
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use diesel::prelude::*;
use serde::Deserialize;
use std::ops::DerefMut;

fn url_exists_for_user(
    db: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<SqliteConnection>>,
    user: &crate::users::models::User,
    url_param: &str,
) -> Result<bool, diesel::result::Error> {
    use crate::schema::faenge::dsl::*;

    let count: i64 = Fang::belonging_to(user)
        .filter(url.eq(url_param))
        .filter(user_id.eq(user.id))
        .count()
        .get_result(db.deref_mut())?;

    if count > 1 {
        log::warn!(
            "URL {:?} saved more than once ({} times) by user {:?}",
            url_param,
            count,
            user
        );
    }

    Ok(count > 0)
}

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
    log::debug!("Received has request: {:?}", payload);

    let mut db = match state.db.get() {
        Ok(conn) => conn,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    match url_exists_for_user(&mut db, &auth_info.0, &payload.url) {
        Ok(true) => StatusCode::FOUND,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(e) => {
            log::error!(
                "Error while checking faenge for: {:?}, error: {:?}",
                &auth_info.0,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ForgetFangReq {
    pub url: String,
}
pub async fn forget(
    State(state): State<AppState>,
    auth_info: AuthInfo,
    Json(payload): Json<ForgetFangReq>,
) -> StatusCode {
    use crate::schema::faenge::dsl::*;

    log::debug!("Received forget request: {:?}", payload);

    let mut db = match state.db.get() {
        Ok(conn) => conn,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let matching_faenge = Fang::belonging_to(&auth_info.0)
        .filter(url.eq(&payload.url))
        .filter(user_id.eq(&auth_info.0.id))
        .select(Fang::as_select())
        .load(&mut db);

    match matching_faenge {
        Ok(res) => match res.len() {
            0 => StatusCode::NOT_FOUND,
            1 => {
                let fang = res.first().unwrap();
                match diesel::delete(faenge.filter(id.eq(fang.id))).execute(&mut db) {
                    Ok(c) => {
                        if c != 1 {
                            log::error!(
                                "Expected one delete row while deleting fang for: {:?}, got: {:?}",
                                &auth_info.0,
                                c,
                            );
                            StatusCode::INTERNAL_SERVER_ERROR
                        } else {
                            StatusCode::OK
                        }
                    }
                    Err(e) => {
                        log::error!(
                            "Db delete error while deleting fang for: {:?}, error: {:?}",
                            &auth_info.0,
                            e
                        );
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                }
            }
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
                "Db list error while deleting fang for: {:?}, error: {:?}",
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

    let mut db = match state.db.get() {
        Ok(conn) => conn,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database connection timeout".to_string(),
            );
        }
    };

    // Check if URL already exists for this user
    match url_exists_for_user(&mut db, &auth_info.0, &payload.url) {
        Ok(true) => {
            // TODO: Should we return something different here?
            return (StatusCode::OK, "already caught".to_string());
        }
        Ok(false) => {
            // Continue with save
        }
        Err(e) => {
            log::error!(
                "Error checking for duplicate URL {:?} for user {:?}: {:?}",
                payload.url,
                auth_info.0,
                e
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "troubles".to_string());
        }
    }

    let new_fang = NewFang::new(payload.url, payload.title, auth_info.0.id);
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
