use crate::schema::users;
use chrono::Utc;
use diesel::prelude::*;
use rand::distr::Alphanumeric;
use rand::Rng;
use serde::Deserialize;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::faenge)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Fang {
    pub id: i32,
    pub url: String,
    pub title: Option<String>,
    pub time_created: String,
}

#[derive(Queryable, Selectable, Debug)]
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
pub struct NewUser {
    pub name: String,
    pub password_hash: String,
    pub api_key: String,
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
