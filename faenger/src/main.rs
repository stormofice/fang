#![allow(clippy::uninlined_format_args)]
use diesel::{Connection, SqliteConnection};
use dotenvy::dotenv;
use std::env;
use std::sync::{Arc, Mutex};

pub mod auth;
pub mod links;
pub mod routes;
pub mod schema;
pub mod users;

fn setup_database() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url).expect("Could not establish sqlite connection")
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<SqliteConnection>>,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();
    dotenv().ok();

    let state = AppState {
        db: Arc::new(Mutex::new(setup_database())),
    };

    let app = routes::create_router().with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4567")
        .await
        .expect("Could not bind tcp listener");

    axum::serve(listener, app)
        .await
        .expect("Axum stopped serving 😤")
}

