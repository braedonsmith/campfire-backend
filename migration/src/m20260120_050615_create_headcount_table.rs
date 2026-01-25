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
                    .table(Headcount::Table)
                    .if_not_exists()
                    .col(pk_auto(Headcount::Id))
                    .col(string(Headcount::Name))
                    .col(string(Headcount::Location))
                    .col(date_time(Headcount::CreatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(HeadcountEntry::Table)
                    .if_not_exists()
                    .col(pk_auto(HeadcountEntry::Id))
                    .col(integer(HeadcountEntry::HeadcountId))
                    .col(integer(HeadcountEntry::CAPID))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-headcount-entry-headcount-id")
                            .from(HeadcountEntry::Table, HeadcountEntry::HeadcountId)
                            .to(Headcount::Table, Headcount::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-headcount-entry-capid")
                            .from(HeadcountEntry::Table, HeadcountEntry::CAPID)
                            .to(Attendee::Table, Attendee::CAPID),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-headcount-id-capid")
                    .table(HeadcountEntry::Table)
                    .col(HeadcountEntry::HeadcountId)
                    .col(HeadcountEntry::CAPID)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .name("idx-headcount-id-capid")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(HeadcountEntry::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(Headcount::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Headcount {
    Table,
    Id,
    Name,
    Location,
    CreatedAt,
}

#[derive(DeriveIden)]
enum HeadcountEntry {
    Table,
    Id,
    HeadcountId,
    CAPID,
}
