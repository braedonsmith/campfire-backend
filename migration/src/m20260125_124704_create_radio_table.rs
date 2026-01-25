use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260108_142456_create_table::Attendee;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RadioType::Table)
                    .if_not_exists()
                    .col(pk_auto(RadioType::Id))
                    .col(string(RadioType::Make))
                    .col(string(RadioType::Model))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Radio::Table)
                    .if_not_exists()
                    .col(pk_auto(Radio::Id))
                    .col(integer(Radio::TypeId))
                    .col(string(Radio::PropertyTagNumber))
                    .col(string(Radio::OwnedByUnit))
                    .col(integer_null(Radio::IssuedTo))
                    .col(boolean(Radio::InService))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-radio-radio-type")
                            .from(Radio::Table, Radio::TypeId)
                            .to(RadioType::Table, RadioType::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-radio-issued-to-capid")
                            .from(Radio::Table, Radio::IssuedTo)
                            .to(Attendee::Table, Attendee::CAPID),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().if_exists().table(Radio::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(RadioType::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RadioType {
    Table,
    Id,
    Make,
    Model,
}

#[derive(DeriveIden)]
enum Radio {
    Table,
    Id,
    TypeId,
    PropertyTagNumber,
    OwnedByUnit,
    IssuedTo,
    InService,
}
