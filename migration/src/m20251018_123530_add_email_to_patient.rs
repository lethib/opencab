use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Patients {
  Table,
  Email,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Patients::Table)
          .add_column(
            ColumnDef::new(Patients::Email)
              .string()
              .not_null()
              .default("default@mail.com"),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(Table::alter().table(Patients::Table).drop_column(Patients::Email).to_owned())
      .await
  }
}
