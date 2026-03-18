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
          .modify_column(ColumnDef::new(Patients::Email).string().null())
          .to_owned(),
      )
      .await?;

    manager
      .get_connection()
      .execute_unprepared("UPDATE patients SET email = NULL WHERE email = 'default@mail.com'")
      .await
      .map(|_| ())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared("UPDATE patients SET email = 'default@mail.com' WHERE email IS NULL")
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Patients::Table)
          .modify_column(
            ColumnDef::new(Patients::Email)
              .string()
              .not_null()
              .default("default@mail.com"),
          )
          .to_owned(),
      )
      .await
  }
}
