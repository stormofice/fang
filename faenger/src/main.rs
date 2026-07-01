#![allow(clippy::uninlined_format_args)]
use diesel::SqliteConnection;
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use serde::Deserialize;
use std::env;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::{Receiver, Sender};

pub mod auth;
pub mod backlinks;
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

#[derive(Debug, Deserialize)]
pub struct FaengerConfig {
    pub allow_signups: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub config: Arc<RwLock<FaengerConfig>>,
    pub backlink_tx: Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init_timed();
    dotenv().ok();

    let config: FaengerConfig = config::Config::builder()
        .add_source(config::File::with_name("faenger.toml").required(true))
        .build()?
        .try_deserialize()?;
    log::debug!("Config: {:?}", &config);

    let (tx, mut rx): (Sender<String>, Receiver<String>) = tokio::sync::mpsc::channel(32);

    tokio::spawn(async move {
        log::info!("Starting backlink resolver");
        while let Some(url) = rx.recv().await {
            log::info!("should resolve new backlink: {url}");
        }
    });

    let state = AppState {
        db: setup_database(),
        config: Arc::new(RwLock::new(config)),
        backlink_tx: tx,
    };

    let app = routes::create_router().with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4567")
        .await
        .expect("Could not bind tcp listener");

    axum::serve(listener, app)
        .await
        .map_err(|err| anyhow::anyhow!(err))
}
