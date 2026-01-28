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
use crate::handlers::headcount::*;
use crate::handlers::radios::*;
use crate::handlers::root::root;
use crate::handlers::uploads::*;
use crate::handlers::vehicles::*;

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

    let debug = match dotenvy::var("DEBUG")
        .expect("DEBUG is not set in .env file")
        .as_str()
    {
        "true" => true,
        _ => false,
    };

    match tokio::fs::try_exists(uploads_path.clone()).await {
        Ok(true) => {}
        Ok(false) => panic!("Broken uploads folder symlink"),
        _ => tokio::fs::create_dir(uploads_path)
            .await
            .expect("Failed to create uploads folder"),
    };

    let state = Arc::new(AppState {
        db: Database::connect(db_url)
            .await
            .expect("Database connection failed"),
        version: env!("CARGO_PKG_VERSION").to_string(),
    });

    Migrator::up(&state.db, None).await?;

    let cors = if debug {
        CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_origin(Any)
    } else {
        CorsLayer::new()
            .allow_credentials(true)
            .allow_methods([Method::GET, Method::POST, Method::DELETE])
            .allow_origin(
                dotenvy::var("CORS_ORIGIN")
                    .expect("CORS_ORIGIN is not set in .env file")
                    .parse::<HeaderValue>()
                    .unwrap(),
            )
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/attendees", get(get_all_attendees))
        .route(
            "/attendees/{id}",
            get(get_attendee_by_capid).delete(delete_attendee),
        )
        .route("/attendees/new", post(create_attendee))
        .route("/attendees/new/bulk", post(create_attendee_bulk))
        .route("/headcounts", get(get_all_headcounts))
        .route(
            "/headcounts/{id}",
            get(get_headcount_by_id).delete(delete_headcount),
        )
        .route(
            "/headcounts/{id}/manage",
            post(add_to_headcount).delete(remove_from_headcount),
        )
        .route("/headcounts/new", post(create_headcount))
        .route("/uploads", get(get_all_uploads))
        .route("/uploads/file", get(download_file))
        .route(
            "/radios",
            get(get_all_radios)
                .post(create_new_radio)
                .delete(delete_radio),
        )
        .route(
            "/radios/types",
            get(get_all_radio_types)
                .post(create_new_radio_type)
                .delete(delete_radio_type),
        )
        .route("/radios/issue", post(issue_radio).delete(return_radio))
        .route(
            "/vehicles",
            get(get_all_vehicles)
                .post(create_new_vehicle)
                .delete(delete_vehicle),
        )
        .route(
            "/vehicles/types",
            get(get_all_vehicle_types)
                .post(create_new_vehicle_type)
                .delete(delete_vehicle_type),
        )
        .route(
            "/vehicles/issue",
            post(issue_vehicle).delete(return_vehicle),
        )
        .route(
            "/vehicles/inspect",
            get(get_all_inspections).post(start_inspection),
        )
        .route(
            "/vehicles/inspect/{id}",
            get(get_inspection)
                .post(update_inspection)
                .delete(delete_inspection),
        )
        .route("/vehicles/inspect/{id}/sign", post(sign_inspection))
        .route("/vehicles/inspect/{id}/ic", post(override_inspection))
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
