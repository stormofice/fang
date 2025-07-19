use crate::users::models::User;
use crate::AppState;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use diesel::prelude::*;

pub struct AuthInfo(pub User);

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
            let mut db = app_state.db.get().map_err(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database connection timeout".to_string())
            })?;
            use crate::schema::users::dsl::*;

            match users
                .filter(api_key.eq(req_api_key))
                .select(User::as_select())
                .load(&mut db)
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