use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Replace the sample below with your own migration scripts
    manager
      .alter_table(
        Table::alter()
          .table(Alias::new("patients"))
          .modify_column(ColumnDef::new(Alias::new("ssn")).string().null())
          .modify_column(ColumnDef::new(Alias::new("hashed_ssn")).string().null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Replace the sample below with your own migration scripts
    manager
      .get_connection()
      .execute_unprepared("UPDATE patients SET ssn = 'empty_ssn' WHERE ssn IS NULL")
      .await?;
    manager
      .get_connection()
      .execute_unprepared(
        "UPDATE patients SET hashed_ssn = 'empty_hashed_ssn' WHERE hashed_ssn IS NULL",
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Alias::new("patients"))
          .modify_column(ColumnDef::new(Alias::new("ssn")).string())
          .modify_column(ColumnDef::new(Alias::new("hashed_ssn")).string())
          .to_owned(),
      )
      .await
  }
}
