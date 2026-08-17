use crate::AppState;
use crate::auth::extractors::AuthInfo;
use crate::backlinks::models::Backlink;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

#[derive(Debug, Deserialize)]
pub struct ResolveBacklinkReq {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ResolveBacklinkResp {
    pub lobsters_links: Option<Vec<String>>,
    pub hn_links: Option<Vec<String>>,
}

fn split_backlinks(links: String) -> Vec<String> {
    links.split("🙂‍↕️").map(str::to_string).collect()
}

pub async fn resolve(
    State(state): State<AppState>,
    _auth_info: AuthInfo,
    Query(payload): Query<ResolveBacklinkReq>,
) -> Result<Json<ResolveBacklinkResp>, StatusCode> {
    use crate::schema::backlinks::dsl::*;

    let mut db = match state.db.get() {
        Ok(conn) => conn,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match backlinks
        .filter(url.eq(&payload.url))
        .load::<Backlink>(db.deref_mut())
    {
        Ok(links) => match links.len() {
            0 => Err(StatusCode::NOT_FOUND),
            1 => {
                let bl = links.first().cloned().unwrap();
                let resp = ResolveBacklinkResp {
                    lobsters_links: bl.lobsters_links.map(split_backlinks),
                    hn_links: bl.hn_links.map(split_backlinks),
                };
                Ok(Json(resp))
            }
            _ => {
                log::warn!("Multiple backlinks found for url??: {}", &payload.url);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(err) => {
            log::error!(
                "Db error while trying to resolve backlink for {}: {:?}",
                payload.url,
                err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
