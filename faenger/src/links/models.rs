use crate::users::models::User;
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Serialize)]
#[diesel(table_name = crate::schema::faenge)]
#[diesel(belongs_to(User))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Fang {
    // (Ab)using skip serde functionality to limit what is returned to users
    #[serde(skip_serializing)]
    pub id: i32,
    pub url: String,
    pub title: Option<String>,
    pub time_created: String,
    #[serde(skip_serializing)]
    pub user_id: i32,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::faenge)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewFang {
    url: String,
    title: Option<String>,
    time_created: String,
    user_id: i32,
}

impl NewFang {
    pub fn new(url: String, title: Option<String>, user_id: i32) -> NewFang {
        NewFang {
            url,
            title,
            time_created: Utc::now().to_rfc3339(),
            user_id,
        }
    }
}