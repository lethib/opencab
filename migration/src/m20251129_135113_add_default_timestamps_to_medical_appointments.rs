use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.alter_table(
      Table::alter()
        .table(MedicalAppointments::Table)
        .modify_column(
          ColumnDef::new(MedicalAppointments::CreatedAt)
            .timestamp_with_time_zone()
            .not_null()
            .default(Expr::current_timestamp()),
        )
        .modify_column(
          ColumnDef::new(MedicalAppointments::UpdatedAt)
            .timestamp_with_time_zone()
            .not_null()
            .default(Expr::current_timestamp()),
        )
        .to_owned(),
    )
    .await
  }

  async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.alter_table(
      Table::alter()
        .table(MedicalAppointments::Table)
        .modify_column(
          ColumnDef::new(MedicalAppointments::CreatedAt)
            .timestamp_with_time_zone()
            .not_null(),
        )
        .modify_column(
          ColumnDef::new(MedicalAppointments::UpdatedAt)
            .timestamp_with_time_zone()
            .not_null(),
        )
        .to_owned(),
    )
    .await
  }
}

#[derive(Iden)]
enum MedicalAppointments {
  Table,
  CreatedAt,
  UpdatedAt,
}
