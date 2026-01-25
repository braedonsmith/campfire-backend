use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use entity::prelude::{Radio, RadioType};
use sea_orm::{ActiveModelTrait, ActiveValue::*, DbErr, EntityTrait};
use serde::Deserialize;

use crate::AppState;

pub(crate) async fn get_all_radios(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let radios = Radio::find()
        .all(&state.db)
        .await
        .expect("Could not get headcounts");

    Json(radios)
}

pub(crate) async fn create_new_radio(
    State(state): State<Arc<AppState>>,
    Json(radio): Json<entity::radio::Model>,
) -> impl IntoResponse {
    let active_model = entity::radio::ActiveModel {
        id: NotSet,
        type_id: Set(radio.type_id),
        property_tag_number: Set(radio.property_tag_number),
        owned_by_unit: Set(radio.owned_by_unit),
        issued_to: Set(None),
        in_service: Set(radio.in_service),
    };

    match active_model.insert(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotInserted) => StatusCode::BAD_REQUEST,
        Err(e) => {
            println!("{e:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub(crate) async fn delete_radio(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = match params.get("id") {
        Some(s) => match s.parse() {
            Ok(v) => v,
            Err(_) => return StatusCode::BAD_REQUEST,
        },
        None => return StatusCode::BAD_REQUEST,
    };

    let model = entity::radio::ActiveModel {
        id: Set(id),
        ..Default::default()
    };

    match model.delete(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotFound(_)) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn get_all_radio_types(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let radio_types = RadioType::find()
        .all(&state.db)
        .await
        .expect("Could not get radio types");

    Json(radio_types)
}

pub(crate) async fn create_new_radio_type(
    State(state): State<Arc<AppState>>,
    Json(radio_type): Json<entity::radio_type::Model>,
) -> impl IntoResponse {
    let active_model = entity::radio_type::ActiveModel {
        id: NotSet,
        make: Set(radio_type.make),
        model: Set(radio_type.model),
    };

    match active_model.insert(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotInserted) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn delete_radio_type(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id = match params.get("id") {
        Some(s) => match s.parse() {
            Ok(v) => v,
            Err(_) => return StatusCode::BAD_REQUEST,
        },
        None => return StatusCode::BAD_REQUEST,
    };

    let model = entity::radio_type::ActiveModel {
        id: Set(id),
        ..Default::default()
    };

    match model.delete(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotFound(_)) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Deserialize)]
pub(crate) struct IssueRadioParameters {
    radio_id: i32,
    capid: i32,
}

pub(crate) async fn issue_radio(
    State(state): State<Arc<AppState>>,
    Json(data): Json<IssueRadioParameters>,
) -> impl IntoResponse {
    let model = match Radio::find_by_id(data.radio_id).one(&state.db).await {
        Ok(opt) => match opt {
            Some(model) => model,
            None => return StatusCode::BAD_REQUEST,
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let active_model = entity::radio::ActiveModel {
        id: Unchanged(model.id),
        type_id: Unchanged(model.type_id),
        property_tag_number: Unchanged(model.property_tag_number),
        owned_by_unit: Unchanged(model.owned_by_unit),
        issued_to: Set(Some(data.capid)),
        in_service: Unchanged(model.in_service),
    };

    match active_model.update(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn return_radio(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id: i32 = match params.get("radioId") {
        Some(s) => match s.parse() {
            Ok(v) => v,
            Err(_) => return StatusCode::BAD_REQUEST,
        },
        None => return StatusCode::BAD_REQUEST,
    };

    let in_service = match params.get("inService") {
        Some(s) => match s.parse() {
            Ok(b) => b,
            Err(_) => return StatusCode::BAD_REQUEST,
        },
        None => true,
    };

    let model = match Radio::find_by_id(id).one(&state.db).await {
        Ok(opt) => match opt {
            Some(model) => model,
            None => return StatusCode::BAD_REQUEST,
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let active_model = entity::radio::ActiveModel {
        id: Unchanged(model.id),
        type_id: Unchanged(model.type_id),
        property_tag_number: Unchanged(model.property_tag_number),
        owned_by_unit: Unchanged(model.owned_by_unit),
        issued_to: Set(None),
        in_service: Set(in_service),
    };

    match active_model.update(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
