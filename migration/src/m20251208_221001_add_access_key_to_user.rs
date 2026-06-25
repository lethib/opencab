use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Users {
  Table,
  AccessKey,
  IsAccessKeyVerified,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Users::Table)
          .add_column(ColumnDef::new(Users::AccessKey).string())
          .add_column(ColumnDef::new(Users::IsAccessKeyVerified).boolean().not_null().default(false))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Users::Table)
          .drop_column(Users::AccessKey)
          .drop_column(Users::IsAccessKeyVerified)
          .to_owned(),
      )
      .await
  }
}
