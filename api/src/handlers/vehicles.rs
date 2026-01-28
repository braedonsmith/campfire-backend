use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use chrono::Local;
use entity::prelude::{Attendee, Vehicle, VehicleInspection, VehicleType};
use sea_orm::{
    ActiveModelTrait, ActiveValue::*, DbErr, EntityLoaderTrait, EntityTrait, IntoActiveModel,
};
use serde::Deserialize;

use crate::AppState;

pub(crate) async fn get_all_vehicles(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let vehicles: Vec<entity::vehicle::ModelEx> = Vehicle::load()
        .with(VehicleType)
        .with(Attendee)
        .all(&state.db)
        .await
        .unwrap();

    Json(vehicles)
}

pub(crate) async fn create_new_vehicle(
    State(state): State<Arc<AppState>>,
    Json(vehicle): Json<entity::vehicle::Model>,
) -> impl IntoResponse {
    let active_model = entity::vehicle::ActiveModel {
        id: Set(vehicle.id),
        type_id: Set(vehicle.type_id),
        year: Set(vehicle.year),
        owned_by_unit: Set(vehicle.owned_by_unit),
        issued_to: Set(None),
        in_service: Set(vehicle.in_service),
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

pub(crate) async fn delete_vehicle(
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

    let model = entity::vehicle::ActiveModel {
        id: Set(id),
        ..Default::default()
    };

    match model.delete(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotFound(_)) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn get_all_vehicle_types(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let vehicle_types = VehicleType::find()
        .all(&state.db)
        .await
        .expect("Could not get vehicle types");

    Json(vehicle_types)
}

pub(crate) async fn create_new_vehicle_type(
    State(state): State<Arc<AppState>>,
    Json(vehicle_type): Json<entity::vehicle_type::Model>,
) -> impl IntoResponse {
    let active_model = entity::vehicle_type::ActiveModel {
        id: NotSet,
        make: Set(vehicle_type.make),
        model: Set(vehicle_type.model),
        capacity: Set(vehicle_type.capacity),
    };

    match active_model.insert(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotInserted) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn delete_vehicle_type(
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

    let model = entity::vehicle_type::ActiveModel {
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
pub(crate) struct IssueVehicleParameters {
    vehicle_id: i32,
    capid: i32,
}

pub(crate) async fn issue_vehicle(
    State(state): State<Arc<AppState>>,
    Json(data): Json<IssueVehicleParameters>,
) -> impl IntoResponse {
    let model = match Vehicle::find_by_id(data.vehicle_id).one(&state.db).await {
        Ok(opt) => match opt {
            Some(model) => model,
            None => return StatusCode::BAD_REQUEST,
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let active_model = entity::vehicle::ActiveModel {
        id: Unchanged(model.id),
        type_id: Unchanged(model.type_id),
        year: Unchanged(model.year),
        owned_by_unit: Unchanged(model.owned_by_unit),
        issued_to: Set(Some(data.capid)),
        in_service: Unchanged(model.in_service),
    };

    match active_model.update(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn return_vehicle(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let id: i32 = match params.get("vehicleId") {
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

    let model = match Vehicle::find_by_id(id).one(&state.db).await {
        Ok(opt) => match opt {
            Some(model) => model,
            None => return StatusCode::BAD_REQUEST,
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let active_model = entity::vehicle::ActiveModel {
        id: Unchanged(model.id),
        type_id: Unchanged(model.type_id),
        year: Unchanged(model.year),
        owned_by_unit: Unchanged(model.owned_by_unit),
        issued_to: Set(None),
        in_service: Set(in_service),
    };

    match active_model.update(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn get_all_inspections(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let inspections = VehicleInspection::find()
        .all(&state.db)
        .await
        .expect("Could not get inspections");

    Json(inspections)
}

#[derive(Deserialize)]
pub(crate) struct StartInspectionParameters {
    vehicle_id: i32,
    capid: i32,
}

pub(crate) async fn start_inspection(
    State(state): State<Arc<AppState>>,
    Json(data): Json<StartInspectionParameters>,
) -> impl IntoResponse {
    let active_model = entity::vehicle_inspection::ActiveModel {
        id: NotSet,
        started_at: Set(Local::now().naive_local()),
        vehicle_id: Set(data.vehicle_id),
        inspector_capid: Set(Some(data.capid)),
        ..Default::default()
    };

    match active_model.insert(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn get_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<entity::vehicle_inspection::ModelEx>, StatusCode> {
    match entity::vehicle_inspection::Entity::load()
        .filter_by_id(id)
        .with(Vehicle)
        .with(Attendee)
        .one(&state.db)
        .await
    {
        Ok(model) => match model {
            Some(v) => Ok(Json(v)),
            None => return Err(StatusCode::NOT_FOUND),
        },
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub(crate) async fn update_inspection(
    State(state): State<Arc<AppState>>,
    Json(inspection): Json<entity::vehicle_inspection::Model>,
) -> impl IntoResponse {
    let mut model = match entity::vehicle_inspection::Entity::load()
        .filter_by_id(inspection.id)
        .one(&state.db)
        .await
    {
        Ok(model) => match model {
            Some(v) => v.into_active_model(),
            None => return StatusCode::BAD_REQUEST,
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    model.windows = Set(inspection.windows);
    model.windows_remarks = Set(inspection.windows_remarks);
    model.headlights = Set(inspection.headlights);
    model.headlights_remarks = Set(inspection.headlights_remarks);
    model.tail_lights = Set(inspection.tail_lights);
    model.tail_lights_remarks = Set(inspection.tail_lights_remarks);
    model.brake_lights = Set(inspection.brake_lights);
    model.brake_lights_remarks = Set(inspection.brake_lights_remarks);
    model.turn_signals = Set(inspection.turn_signals);
    model.turn_signals_remarks = Set(inspection.turn_signals_remarks);
    model.emergency_lights = Set(inspection.emergency_lights);
    model.emergency_lights_remarks = Set(inspection.emergency_lights_remarks);
    model.license_plate_light = Set(inspection.license_plate_light);
    model.license_plate_light_remarks = Set(inspection.license_plate_light_remarks);
    model.backup_light = Set(inspection.backup_light);
    model.backup_light_remarks = Set(inspection.backup_light_remarks);
    model.backup_alarm = Set(inspection.backup_alarm);
    model.backup_alarm_remarks = Set(inspection.backup_alarm_remarks);
    model.wiper_blades = Set(inspection.wiper_blades);
    model.wiper_blades_remarks = Set(inspection.wiper_blades_remarks);
    model.horn = Set(inspection.horn);
    model.horn_remarks = Set(inspection.horn_remarks);
    model.seats = Set(inspection.seats);
    model.seats_remarks = Set(inspection.seats_remarks);
    model.restraints = Set(inspection.restraints);
    model.restraints_remarks = Set(inspection.restraints_remarks);
    model.mirrors = Set(inspection.mirrors);
    model.mirrors_remarks = Set(inspection.mirrors_remarks);
    model.beacon_light = Set(inspection.beacon_light);
    model.beacon_light_remarks = Set(inspection.beacon_light_remarks);
    model.wiring = Set(inspection.wiring);
    model.wiring_remarks = Set(inspection.wiring_remarks);
    model.brakes = Set(inspection.brakes);
    model.brakes_remarks = Set(inspection.brakes_remarks);
    model.battery = Set(inspection.battery);
    model.battery_remarks = Set(inspection.battery_remarks);
    model.brake_fluid = Set(inspection.brake_fluid);
    model.brake_fluid_remarks = Set(inspection.brake_fluid_remarks);
    model.exhaust_system = Set(inspection.exhaust_system);
    model.exhaust_system_remarks = Set(inspection.exhaust_system_remarks);
    model.oil_level = Set(inspection.oil_level);
    model.oil_level_remarks = Set(inspection.oil_level_remarks);
    model.oil_last_change = Set(inspection.oil_last_change);
    model.coolant = Set(inspection.coolant);
    model.coolant_remarks = Set(inspection.coolant_remarks);
    model.belts_hoses = Set(inspection.belts_hoses);
    model.belts_hoses_remarks = Set(inspection.belts_hoses_remarks);
    model.transmission = Set(inspection.transmission);
    model.transmission_remarks = Set(inspection.transmission_remarks);
    model.battery_cables = Set(inspection.battery_cables);
    model.battery_cables_remarks = Set(inspection.battery_cables_remarks);
    model.air_filter = Set(inspection.air_filter);
    model.air_filter_remarks = Set(inspection.air_filter_remarks);
    model.body = Set(inspection.body);
    model.body_remarks = Set(inspection.body_remarks);
    model.paint = Set(inspection.paint);
    model.paint_remarks = Set(inspection.paint_remarks);
    model.bumpers = Set(inspection.bumpers);
    model.bumpers_remarks = Set(inspection.bumpers_remarks);
    model.tire_tread = Set(inspection.tire_tread);
    model.tire_tread_remarks = Set(inspection.tire_tread_remarks);
    model.tire_gauge_present = Set(inspection.tire_gauge_present);
    model.front_recommended_pressure = Set(inspection.front_recommended_pressure);
    model.rear_recommended_pressure = Set(inspection.rear_recommended_pressure);
    model.left_front_pressure = Set(inspection.left_front_pressure);
    model.right_front_pressure = Set(inspection.right_front_pressure);
    model.left_rear_pressure = Set(inspection.left_rear_pressure);
    model.right_rear_pressure = Set(inspection.right_rear_pressure);
    model.logbook_present = Set(inspection.logbook_present);
    model.registration_present = Set(inspection.registration_present);
    model.registration_present_remarks = Set(inspection.registration_present_remarks);
    model.registration_current = Set(inspection.registration_current);
    model.registration_current_remarks = Set(inspection.registration_current_remarks);
    model.insurance_present = Set(inspection.insurance_present);
    model.insurance_present_remarks = Set(inspection.insurance_present_remarks);
    model.insurance_current = Set(inspection.insurance_current);
    model.insurance_current_remarks = Set(inspection.insurance_current_remarks);
    model.capf132_present = Set(inspection.capf132_present);
    model.capf132_present_remarks = Set(inspection.capf132_present_remarks);
    model.capf132_current = Set(inspection.capf132_current);
    model.capf132_current_remarks = Set(inspection.capf132_current_remarks);
    model.capf132_signed = Set(inspection.capf132_signed);
    model.capf132_signed_remarks = Set(inspection.capf132_signed_remarks);
    model.first_aid_kit_present = Set(inspection.first_aid_kit_present);
    model.first_aid_kit_present_remarks = Set(inspection.first_aid_kit_present_remarks);
    model.toolkit_present = Set(inspection.toolkit_present);
    model.toolkit_present_remarks = Set(inspection.toolkit_present_remarks);
    model.toolkit_secured = Set(inspection.toolkit_secured);
    model.toolkit_secured_remarks = Set(inspection.toolkit_secured_remarks);
    model.survival_kit_present = Set(inspection.survival_kit_present);
    model.survival_kit_present_remarks = Set(inspection.survival_kit_present_remarks);
    model.survival_kit_current = Set(inspection.survival_kit_current);
    model.is_mission_ready = Set(inspection.is_mission_ready);
    model.inspector_capid = Set(inspection.inspector_capid);

    match model.update(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotUpdated) => StatusCode::BAD_REQUEST,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Deserialize)]
pub(crate) struct SignInspectionParameters {
    capid: i32,
}

pub(crate) async fn sign_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(data): Json<SignInspectionParameters>,
) -> impl IntoResponse {
    let mut model = match entity::vehicle_inspection::Entity::load()
        .filter_by_id(id)
        .one(&state.db)
        .await
    {
        Ok(model) => match model {
            Some(v) => v.into_active_model(),
            None => return StatusCode::BAD_REQUEST,
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    model.inspector_capid = Set(Some(data.capid));
    model.signed_at = Set(Some(Local::now().naive_local()));

    match model.update(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotUpdated) => StatusCode::BAD_REQUEST,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn override_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(data): Json<SignInspectionParameters>,
) -> impl IntoResponse {
    let mut model = match entity::vehicle_inspection::Entity::load()
        .filter_by_id(id)
        .one(&state.db)
        .await
    {
        Ok(model) => match model {
            Some(v) => v.into_active_model(),
            None => return StatusCode::BAD_REQUEST,
        },
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    model.ic_capid = Set(Some(data.capid));
    model.ic_signed_at = Set(Some(Local::now().naive_local()));

    match model.update(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotUpdated) => StatusCode::BAD_REQUEST,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub(crate) async fn delete_inspection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match entity::vehicle_inspection::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(DbErr::RecordNotFound(_)) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
