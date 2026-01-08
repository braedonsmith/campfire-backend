use std::sync::Arc;

use axum::extract::State;
use axum::response::IntoResponse;

use crate::handlers::AppState;

pub(crate) async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    format!("This is CampFIRE v{}", state.version)
}