//! Database migrations for settings service

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241210_000001_create_cti_registry::Migration),
            Box::new(m20241210_000002_create_settings::Migration),
        ]
    }
}

mod m20241210_000001_create_cti_registry {
    use super::*;

    #[derive(DeriveMigrationName)]
    pub struct Migration;

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .create_table(
                    Table::create()
                        .table(CtiRegistry::Table)
                        .if_not_exists()
                        .col(
                            ColumnDef::new(CtiRegistry::Type)
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(ColumnDef::new(CtiRegistry::Traits).json().not_null())
                        .col(ColumnDef::new(CtiRegistry::Schema).json())
                        .col(
                            ColumnDef::new(CtiRegistry::CreatedAt)
                                .timestamp_with_time_zone()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .col(
                            ColumnDef::new(CtiRegistry::UpdatedAt)
                                .timestamp_with_time_zone()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .to_owned(),
                )
                .await
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .drop_table(Table::drop().table(CtiRegistry::Table).to_owned())
                .await
        }
    }

    #[derive(DeriveIden)]
    enum CtiRegistry {
        Table,
        Type,
        Traits,
        Schema,
        CreatedAt,
        UpdatedAt,
    }
}

mod m20241210_000002_create_settings {
    use super::*;

    #[derive(DeriveMigrationName)]
    pub struct Migration;

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .create_table(
                    Table::create()
                        .table(Settings::Table)
                        .if_not_exists()
                        .col(ColumnDef::new(Settings::Type).string().not_null())
                        .col(ColumnDef::new(Settings::TenantId).uuid().not_null())
                        .col(
                            ColumnDef::new(Settings::DomainObjectId)
                                .string()
                                .not_null(),
                        )
                        .col(ColumnDef::new(Settings::Data).json().not_null())
                        .col(
                            ColumnDef::new(Settings::CreatedAt)
                                .timestamp_with_time_zone()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .col(
                            ColumnDef::new(Settings::UpdatedAt)
                                .timestamp_with_time_zone()
                                .not_null()
                                .default(Expr::current_timestamp()),
                        )
                        .col(ColumnDef::new(Settings::DeletedAt).timestamp_with_time_zone())
                        .primary_key(
                            Index::create()
                                .col(Settings::Type)
                                .col(Settings::TenantId)
                                .col(Settings::DomainObjectId),
                        )
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_settings_gts_type")
                                .from(Settings::Table, Settings::Type)
                                .to(CtiRegistry::Table, CtiRegistry::Type)
                                .on_delete(ForeignKeyAction::Restrict)
                                .on_update(ForeignKeyAction::Cascade),
                        )
                        .to_owned(),
                )
                .await?;

            // Create indexes
            manager
                .create_index(
                    Index::create()
                        .name("idx_settings_type")
                        .table(Settings::Table)
                        .col(Settings::Type)
                        .to_owned(),
                )
                .await?;

            manager
                .create_index(
                    Index::create()
                        .name("idx_settings_tenant_id")
                        .table(Settings::Table)
                        .col(Settings::TenantId)
                        .to_owned(),
                )
                .await?;

            manager
                .create_index(
                    Index::create()
                        .name("idx_settings_domain_object_id")
                        .table(Settings::Table)
                        .col(Settings::DomainObjectId)
                        .to_owned(),
                )
                .await?;

            Ok(())
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .drop_table(Table::drop().table(Settings::Table).to_owned())
                .await
        }
    }

    #[derive(DeriveIden)]
    enum Settings {
        Table,
        Type,
        TenantId,
        DomainObjectId,
        Data,
        CreatedAt,
        UpdatedAt,
        DeletedAt,
    }

    #[derive(DeriveIden)]
    enum CtiRegistry {
        Table,
        Type,
    }
}
