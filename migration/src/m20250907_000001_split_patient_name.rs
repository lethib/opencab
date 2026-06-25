use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.alter_table(
      Table::alter()
        .table(Alias::new("patients"))
        .add_column(ColumnDef::new(Alias::new("first_name")).string().not_null())
        .add_column(ColumnDef::new(Alias::new("last_name")).string().not_null())
        .drop_column(Alias::new("name"))
        .to_owned(),
    )
    .await
  }

  async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.alter_table(
      Table::alter()
        .table(Alias::new("patients"))
        .add_column(ColumnDef::new(Alias::new("name")).string().not_null())
        .drop_column(Alias::new("first_name"))
        .drop_column(Alias::new("last_name"))
        .to_owned(),
    )
    .await
  }
}
