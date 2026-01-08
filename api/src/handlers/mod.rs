use sea_orm::DatabaseConnection;

pub(crate) mod root;

pub(crate) struct AppState {
    pub(crate) db: DatabaseConnection,
    pub(crate) version: String
}