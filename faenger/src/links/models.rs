use crate::users::models::User;
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
    pub time_created: String,
    pub title: Option<String>,
    pub url: String,
    #[serde(skip_serializing)]
    pub user_id: i32,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = crate::schema::faenge)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewFang {
    time_created: String,
    title: Option<String>,
    url: String,
    user_id: i32,
}

impl NewFang {
    pub fn new(title: Option<String>, url: String, user_id: i32) -> NewFang {
        NewFang {
            time_created: chrono::Utc::now().to_rfc3339(),
            title,
            url,
            user_id,
        }
    }
}
