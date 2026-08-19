use crate::AppState;
use crate::auth::extractors::AuthInfo;
use crate::backlinks::split_backlinks;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ResolveBacklinkReq {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ResolveBacklinkResp {
    pub lobsters_links: Option<Vec<String>>,
    pub hn_links: Option<Vec<String>>,
}

pub async fn resolve(
    State(state): State<AppState>,
    _auth_info: AuthInfo,
    Query(payload): Query<ResolveBacklinkReq>,
) -> Result<Json<ResolveBacklinkResp>, StatusCode> {
    match state
        .backlink_resolver
        .retrieve_backlinks(&payload.url)
        .await
    {
        Ok(backlinks) => {
            if let Some(backlinks) = backlinks {
                let resp = ResolveBacklinkResp {
                    lobsters_links: backlinks.lobsters_links.map(split_backlinks),
                    hn_links: backlinks.hn_links.map(split_backlinks),
                };
                Ok(Json(resp))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(err) => {
            log::error!(
                "error while trying to handle backlink resolve request for {}: {}",
                &payload.url,
                err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
