use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.drop_table(Table::drop().table(PatientUsers::Table).to_owned()).await
  }

  async fn down(&self, _m: &SchemaManager) -> Result<(), DbErr> {
    Err(DbErr::Migration(
      "Cannot rollback: patient_users table removal is irreversible".to_string(),
    ))
  }
}

#[derive(DeriveIden)]
enum PatientUsers {
  Table,
}
