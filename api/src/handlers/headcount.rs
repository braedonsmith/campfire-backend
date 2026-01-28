use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use entity::headcount_entry;
use entity::prelude::{Attendee, Headcount, HeadcountEntry};
use sea_orm::EntityLoaderTrait;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, DbErr, EntityTrait, ExprTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

pub(crate) async fn get_all_headcounts(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let headcounts = Headcount::find()
        .all(&state.db)
        .await
        .expect("Could not get headcounts");

    Json(headcounts)
}

pub(crate) async fn get_headcount_by_id(
    State(state): State<Arc<AppState>>,
    Path(capid): Path<i32>,
) -> Result<Json<entity::headcount::ModelEx>, StatusCode> {
    let result = entity::headcount::Entity::load()
        .filter_by_id(capid)
        .with(Attendee)
        .one(&state.db)
        .await;

    match result {
        Ok(opt) => match opt {
            Some(model) => Ok(Json(model)),
            None => Err(StatusCode::NOT_FOUND)
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub(crate) async fn delete_headcount(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let model = entity::headcount::ActiveModel {
        id: Set(id),
        ..Default::default()
    };

    match model.delete(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotFound(_)) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn create_headcount(
    State(state): State<Arc<AppState>>,
    headcount: Json<entity::headcount::Model>,
) -> impl IntoResponse {
    let active_model = entity::headcount::ActiveModel {
        id: NotSet,
        name: Set(headcount.0.name),
        location: Set(headcount.0.location),
        created_at: Set(headcount.0.created_at),
    };

    match active_model.insert(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotInserted) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CAPID {
    capid: i32,
}

pub(crate) async fn add_to_headcount(
    State(state): State<Arc<AppState>>,
    Path(headcount_id): Path<i32>,
    Json(data): Json<CAPID>,
) -> impl IntoResponse {
    let active_model = entity::headcount_entry::ActiveModel {
        id: NotSet,
        headcount_id: Set(headcount_id),
        capid: Set(data.capid),
    };

    match active_model.insert(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotInserted) => StatusCode::BAD_REQUEST,
        Err(DbErr::Query(_)) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn remove_from_headcount(
    State(state): State<Arc<AppState>>,
    Path(headcount_id): Path<i32>,
    Query(data): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let capid = match data.get("capid") {
        Some(string) => match string.parse::<i32>() {
            Ok(c) => c,
            Err(_) => return StatusCode::BAD_REQUEST,
        },
        None => return StatusCode::BAD_REQUEST,
    };

    match HeadcountEntry::delete_many()
        .filter(
            headcount_entry::Column::Capid
                .eq(capid)
                .and(headcount_entry::Column::HeadcountId.eq(headcount_id)),
        )
        .exec(&state.db)
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotFound(_)) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
