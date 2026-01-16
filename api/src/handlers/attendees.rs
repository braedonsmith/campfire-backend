use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use axum::{Form, Json};
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use calamine::{DataType, HeaderRow, Reader, Xlsx, open_workbook};
use chrono::NaiveDate;
use entity::prelude::Attendee;
use sea_orm::{ActiveValue::Set, DbConn, DbErr};
use sea_orm::prelude::{ActiveModelTrait, EntityTrait};

use crate::AppState;

pub(crate) async fn get_all_attendees(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let attendees = Attendee::find().all(&state.db).await.expect("Could not get attendees");

    Json(attendees)
}

pub(crate) async fn get_attendee_by_capid(State(state): State<Arc<AppState>>, Path(capid): Path<i32>) -> impl IntoResponse {
    let attendee = Attendee::find_by_id(capid)
        .one(&state.db)
        .await
        .expect(format!("Could not find attendee for CAPID {capid}").as_str())
        .unwrap_or_else(|| panic!("Could not find attendee for CAPID {capid}"));

    Json(attendee)
}

pub(crate) async fn create_attendee(State(state): State<Arc<AppState>>, form: Form<entity::attendee::Model>) -> impl IntoResponse {
    let attendee = form.0;

    match create(&state.db, attendee).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST
    }
}

pub(crate) async fn create_attendee_bulk(State(state): State<Arc<AppState>>, mut multipart: Multipart) -> impl IntoResponse {
    let mut file_path = String::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        match content_type.as_str() {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                let mut path = PathBuf::from("uploads");
                path.push(format!("{}_{file_name}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis()));

                match tokio::fs::write(&path, data).await {
                    Ok(_) => {},
                    Err(_) => return StatusCode::BAD_REQUEST
                }

                file_path = path.to_string_lossy().to_string();
            },
            _ => return StatusCode::BAD_REQUEST
        }
    }

    let mut excel: Xlsx<_> = open_workbook(file_path).unwrap();
    let sheet = excel.with_header_row(HeaderRow::FirstNonEmptyRow).worksheet_range("UniversalReport").unwrap();

    let rows = sheet.rows();

    for row in rows {
        let attendee = entity::attendee::Model {
            capid: row[1].get_int().unwrap() as i32,
            rank: row[5].get_string().unwrap().to_string(),
            last_name: row[6].get_string().unwrap().to_string(),
            first_name: row[7].get_string().unwrap().to_string(),
            middle_name: row[8].get_string().map(str::to_string),
            unit: format!("{}-{}-{:0>3}", row[9].get_string().unwrap(), row[10].get_string().unwrap(), row[11].get_int().unwrap()),
            gender: row[11].get_string().unwrap().to_string(),
            date_of_birth: {
                let (year, month, _, _, _, _, _) = row[12].get_datetime().unwrap().to_ymd_hms_milli();
                NaiveDate::parse_from_str(format!("{year}-{month}-01").as_str(), "%Y-%0m-%0d").unwrap()
            },
            age_at_start: row[14].get_int().unwrap() as i32,
            age_at_end: row[15].get_int().unwrap() as i32,
            height: match row[16].get_int().unwrap() {
                0 => None,
                v => Some(v as i32)
            },
            weight: match row[17].get_int().unwrap() {
                0 => None,
                v => Some(v as i32)
            },
            shirt_size: row[18].get_string().map(str::to_string),
            member_type: row[19].get_string().unwrap().to_string(),
            expiration: {
                let (year, month, day, _, _, _, _) = row[20].get_datetime().unwrap().to_ymd_hms_milli();
                NaiveDate::parse_from_str(format!("{year}-{month}-{day}").as_str(), "%Y-%0m-%0d").unwrap()
            },
            member_status: row[21].get_string().unwrap().to_string(),
            home_phone: row[22].get_string().map(str::to_string),
            cell_phone: row[23].get_string().map(str::to_string),
            email: row[26].get_string().unwrap().to_string(),
            address1: row[27].get_string().unwrap().to_string(),
            address2: row[28].get_string().unwrap().to_string(),
            city: row[29].get_string().unwrap().to_string(),
            state: row[30].get_string().unwrap().to_string(),
            zip_code: row[31].get_string().unwrap().to_string(),
            registration_status: row[34].get_string().unwrap().to_string(),
            is_staff: match row[35].get_string().unwrap() {
                "Yes" => true,
                _ => false
            },
            registration_id: row[39].get_int().unwrap() as i32,
            comments: row[41].get_string().map(str::to_string),
            emergency_contact_name: row[24].get_string().map(str::to_string),
            emergency_contact_number: row[25].get_string().map(str::to_string),
            cadet_parent_phone_primary: row[52].get_string().map(str::to_string),
            cadet_parent_phone_secondary: row[53].get_string().map(str::to_string),
            cadet_parent_email_primary: row[56].get_string().map(str::to_string),
            cadet_parent_email_secondary: row[56].get_string().map(str::to_string),
            unit_commander_name: row[58].get_string().unwrap().to_string(),
            unit_commander_email: row[59].get_string().unwrap().to_string(),
            wing_commander_name: row[60].get_string().unwrap().to_string(),
            wing_commander_email: row[61].get_string().unwrap().to_string(),
            is_pilot: match row[62].get_string().unwrap() {
                "Yes" => true,
                _ => false
            },
            dl_expiration: row[63].get_string().map(|date| NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap()),
            last_encampment: row[64].get_string().map(|date| NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap()),
            highest_o_ride: row[65].get_string().unwrap().parse().unwrap(),
            aircraft_ground_handling: match row[67].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            wing_runner: match row[68].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            orm_basic: match row[69].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            orm_intermediate: match row[70].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            cppt_expiration: match row[71].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            monthly_safety: match row[72].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            icut: match row[73].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            is100: match row[74].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            is700: match row[75].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            capt116: match row[76].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            capt117_part1: match row[77].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            capt117_part2: match row[78].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            capt117_part3: match row[79].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            first_aid: match row[80].get_string().unwrap() {
                "Not Complete" => None,
                date => Some(NaiveDate::parse_from_str(date, "%0d %b %Y").unwrap())
            },
            invoice_id: row[81].get_int().map(|v| v as i32),
            prices_id: row[82].get_int().map(|v| v as i32),
            invoice_status: row[83].get_string().map(str::to_string),
            registered_by: row[86].get_string().map(str::to_string),
        };

        match create(&state.db, attendee).await {
            Ok(_) => {},
            Err(_) => return StatusCode::BAD_REQUEST
        }
    }

    StatusCode::OK
}

async fn create(db: &DbConn, attendee: entity::attendee::Model) -> Result<(), DbErr> {
    let model = entity::attendee::ActiveModel {
        capid: Set(attendee.capid),
        rank: Set(attendee.rank),
        last_name: Set(attendee.last_name),
        first_name: Set(attendee.first_name),
        middle_name: Set(attendee.middle_name),
        unit: Set(attendee.unit),
        gender: Set(attendee.gender),
        date_of_birth: Set(attendee.date_of_birth),
        age_at_start: Set(attendee.age_at_start),
        age_at_end: Set(attendee.age_at_end),
        height: Set(attendee.height),
        weight: Set(attendee.weight),
        shirt_size: Set(attendee.shirt_size),
        member_type: Set(attendee.member_type),
        expiration: Set(attendee.expiration),
        member_status: Set(attendee.member_status),
        home_phone: Set(attendee.home_phone),
        cell_phone: Set(attendee.cell_phone),
        email: Set(attendee.email),
        address1: Set(attendee.address1),
        address2: Set(attendee.address2),
        city: Set(attendee.city),
        state: Set(attendee.state),
        zip_code: Set(attendee.zip_code),
        registration_status: Set(attendee.registration_status),
        is_staff: Set(attendee.is_staff),
        registration_id: Set(attendee.registration_id),
        comments: Set(attendee.comments),
        emergency_contact_name: Set(attendee.emergency_contact_name),
        emergency_contact_number: Set(attendee.emergency_contact_number),
        cadet_parent_phone_primary: Set(attendee.cadet_parent_phone_primary),
        cadet_parent_phone_secondary: Set(attendee.cadet_parent_phone_secondary),
        cadet_parent_email_primary: Set(attendee.cadet_parent_email_primary),
        cadet_parent_email_secondary: Set(attendee.cadet_parent_email_secondary),
        unit_commander_name: Set(attendee.unit_commander_name),
        unit_commander_email: Set(attendee.unit_commander_email),
        wing_commander_name: Set(attendee.wing_commander_name),
        wing_commander_email: Set(attendee.wing_commander_email),
        is_pilot: Set(attendee.is_pilot),
        dl_expiration: Set(attendee.dl_expiration),
        last_encampment: Set(attendee.last_encampment),
        highest_o_ride: Set(attendee.highest_o_ride),
        aircraft_ground_handling: Set(attendee.aircraft_ground_handling),
        wing_runner: Set(attendee.wing_runner),
        orm_basic: Set(attendee.orm_basic),
        orm_intermediate: Set(attendee.orm_intermediate),
        cppt_expiration: Set(attendee.cppt_expiration),
        monthly_safety: Set(attendee.monthly_safety),
        icut: Set(attendee.icut),
        is100: Set(attendee.is100),
        is700: Set(attendee.is700),
        capt116: Set(attendee.capt116),
        capt117_part1: Set(attendee.capt117_part1),
        capt117_part2: Set(attendee.capt117_part2),
        capt117_part3: Set(attendee.capt117_part3),
        first_aid: Set(attendee.first_aid),
        invoice_id: Set(attendee.invoice_id),
        prices_id: Set(attendee.prices_id),
        invoice_status: Set(attendee.invoice_status),
        registered_by: Set(attendee.registered_by),
    };

    match model.save(db).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}