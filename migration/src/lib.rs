pub use sea_orm_migration::prelude::*;

mod m20260108_142456_create_table;
mod m20260120_050615_create_headcount_table;
mod m20260125_124704_create_radio_table;
mod m20260127_184142_create_vehicle_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260108_142456_create_table::Migration),
            Box::new(m20260120_050615_create_headcount_table::Migration),
            Box::new(m20260125_124704_create_radio_table::Migration),
            Box::new(m20260127_184142_create_vehicle_table::Migration),
        ]
    }
}
