use crate::schema::users;
use chrono::Utc;
use diesel::prelude::*;
use rand::Rng;
use rand::distr::Alphanumeric;
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

#[derive(Queryable, Selectable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub password_hash: String,
    pub api_key: String,
    pub time_registered: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewUser {
    name: String,
    password_hash: String,
    api_key: String,
    time_registered: String,
}

impl NewUser {
    pub fn new(name: String, password_hash: String) -> NewUser {
        NewUser {
            name,
            password_hash,
            api_key: rand::rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect(),
            time_registered: Utc::now().to_rfc3339(),
        }
    }
}
