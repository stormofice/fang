use crate::AppState;
use crate::auth::extractors::AuthInfo;
use crate::links::models::{Fang, NewFang};
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use diesel::prelude::*;
use serde::Deserialize;
use std::ops::DerefMut;
use std::time::Duration;
fn get_fang_for_user(
    db: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<SqliteConnection>>,
    user: &crate::users::models::User,
    url_param: &str,
) -> Result<Option<Fang>, diesel::result::Error> {
    use crate::schema::faenge::dsl::*;

    let urls: Vec<Fang> = Fang::belonging_to(user)
        .filter(url.eq(url_param))
        .filter(user_id.eq(user.id))
        .load(db.deref_mut())?;

    if urls.len() > 1 {
        log::warn!(
            "URL {:?} saved more than once ({} times) by user {:?}",
            url_param,
            urls.len(),
            user
        );
    }

    if urls.is_empty() {
        Ok(None)
    } else {
        Ok(Some(urls.first().cloned().unwrap()))
    }
}

fn set_soft_delete_for_fang(
    db: &mut r2d2::PooledConnection<diesel::r2d2::ConnectionManager<SqliteConnection>>,
    fang: &Fang,
    delete: bool,
) -> anyhow::Result<()> {
    use crate::schema::faenge::dsl::faenge;
    use crate::schema::faenge::{id, soft_delete};

    match diesel::update(faenge.filter(id.eq(fang.id)))
        .set(soft_delete.eq(delete))
        .execute(db)
    {
        Ok(c) => {
            if c != 1 {
                log::error!(
                    "Expected one update row while changing soft_delete for: \
                                {:?}, \
                                got: \
                                {:?}",
                    fang,
                    c,
                );
                Err(anyhow::anyhow!(
                    "Affected wrong number of rows during soft delete"
                ))
            } else {
                Ok(())
            }
        }
        Err(e) => {
            log::error!(
                "Db update error while setting soft_delete for fang: {:?}, error: {:?}",
                fang,
                e
            );

            Err(anyhow::anyhow!("Uhh db has issues :("))
        }
    }
}

pub async fn list(
    State(state): State<AppState>,
    auth_info: AuthInfo,
) -> (StatusCode, Json<Vec<Fang>>) {
    use crate::schema::faenge::dsl::*;
    let mut db = match state.db.get() {
        Ok(conn) => conn,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
    };
    match Fang::belonging_to(&auth_info.0)
        .filter(soft_delete.eq(false))
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

    match get_fang_for_user(&mut db, &auth_info.0, &payload.url) {
        Ok(Some(fang)) => {
            if fang.soft_delete {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::FOUND
            }
        }
        Ok(None) => StatusCode::NOT_FOUND,
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
                match set_soft_delete_for_fang(&mut db, fang, true) {
                    Ok(_) => StatusCode::OK,
                    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
                "Db list error while updating fang for: {:?}, error: {:?}",
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
    match get_fang_for_user(&mut db, &auth_info.0, &payload.url) {
        Ok(Some(fang)) => {
            return if fang.soft_delete {
                match set_soft_delete_for_fang(&mut db, &fang, false) {
                    Ok(_) => (StatusCode::OK, "caught again".to_string()),
                    Err(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Sorry something broke!".to_string(),
                    ),
                }
            } else {
                // TODO: Should we return something different here?
                (StatusCode::OK, "already caught".to_string())
            };
        }
        Ok(None) => {
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

    match state
        .backlink_tx
        .send_timeout(payload.url.clone(), Duration::from_millis(50))
        .await
    {
        Ok(_) => {}
        Err(err) => {
            log::error!("URL backlink resolve backlog full: {:?}", err);
        }
    }

    let new_fang = NewFang::new(payload.title, payload.url, auth_info.0.id);
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
