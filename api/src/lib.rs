use std::env;
use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::AppState;
use crate::handlers::attendees::*;
use crate::handlers::root::root;

mod handlers;

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }

    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let bind_addr = dotenvy::var("BIND_ADDR").expect("BIND_ADDR is not set in .env file");
    let uploads_path = dotenvy::var("UPLOADS_PATH").expect("UPLOADS_PATH is not set in .env file");

    let debug = match dotenvy::var("DEBUG").expect("DEBUG is not set in .env file").as_str() {
        "true" => true,
        _ => false
    };

    match tokio::fs::try_exists(uploads_path.clone()).await {
        Ok(true) => {},
        Ok(false) => panic!("Broken uploads folder symlink"),
        _ => tokio::fs::create_dir(uploads_path).await.expect("Failed to create uploads folder")
    };

    let state = Arc::new(AppState {
        db: Database::connect(db_url).await.expect("Database connection failed"),
        version: env!("CARGO_PKG_VERSION").to_string()
    });

    Migrator::up(&state.db, None).await.unwrap();

    let cors = if debug {
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_origin(Any)
    } else {
        CorsLayer::new()
            .allow_credentials(true)
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_origin(dotenvy::var("CORS_ORIGIN").expect("CORS_ORIGIN is not set in .env file").parse::<HeaderValue>().unwrap())
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/attendees", get(get_all_attendees))
        .route("/attendees/{id}", get(get_attendee_by_capid).delete(delete_attendee))
        .route("/attendees/new", post(create_attendee))
        .route("/attendees/new/bulk", post(create_attendee_bulk))
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
