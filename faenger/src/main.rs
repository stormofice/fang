#![allow(clippy::uninlined_format_args)]
use diesel::r2d2::ConnectionManager;
use diesel::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub mod auth;
pub mod links;
pub mod routes;
pub mod schema;
pub mod users;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

fn setup_database() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Could not create database pool")
}

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();
    dotenv().ok();

    let state = AppState {
        db: setup_database(),
    };

    let app = routes::create_router().with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4567")
        .await
        .expect("Could not bind tcp listener");

    axum::serve(listener, app)
        .await
        .expect("Axum stopped serving 😤")
}
