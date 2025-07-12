use std::env;

use crate::models::{NewUser, User};

use diesel::SqliteConnection;
use diesel::prelude::*;
use dotenvy::dotenv;

pub mod models;
pub mod schema;

fn setup_database() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url).expect("Could not establish sqlite connection")
}

fn main() {
    let mut conn = &mut setup_database();
    use self::schema::users::dsl::*;
    use crate::schema::users;

    let results = users
        .select(User::as_select())
        .load(conn)
        .expect("Could not load users");
    for r in results {
        println!("{:?}", r);
    }

    let user_ret = diesel::insert_into(users::table)
        .values(&NewUser {
            name: "testuser",
            password_hash: "213123",
        })
        .execute(conn);
    println!("Insert: {:?}", user_ret);
}
