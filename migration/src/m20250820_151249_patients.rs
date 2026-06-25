use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Patients {
  Table,
  Id,
  Name,
  Ssn,
  Pid,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Patients::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Patients::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(ColumnDef::new(Patients::Name).string().not_null())
          .col(ColumnDef::new(Patients::Ssn).string().not_null().unique_key())
          .col(ColumnDef::new(Patients::Pid).uuid().not_null().unique_key())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(Table::drop().table(Patients::Table).to_owned()).await
  }
}
