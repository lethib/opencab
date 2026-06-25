use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.alter_table(
      Table::alter()
        .table(Alias::new("patients"))
        .add_column(ColumnDef::new(Alias::new("hashed_ssn")).string().unique_key().not_null())
        .to_owned(),
    )
    .await
  }

  async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.alter_table(
      Table::alter()
        .table(Alias::new("patients"))
        .drop_column(Alias::new("hashed_ssn"))
        .to_owned(),
    )
    .await
  }
}
