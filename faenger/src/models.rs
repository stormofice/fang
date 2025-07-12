use crate::schema::users;
use diesel::prelude::*;
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
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub password_hash: String,
}
