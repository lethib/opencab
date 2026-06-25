use sea_orm_migration::{prelude::*, sea_orm::Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(Patients::Table)
          .add_column(
            ColumnDef::new(Patients::UserId).integer().null(), // Nullable initially, will be set to NOT NULL after populating
          )
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Patients::Table)
          .add_foreign_key(
            TableForeignKey::new()
              .name("fk_patients_user_id")
              .from_tbl(Patients::Table)
              .from_col(Patients::UserId)
              .to_tbl(Users::Table)
              .to_col(Users::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    let populate_sql = r#"
      UPDATE patients p
      SET user_id = pu.user_id
      FROM patient_users pu
      WHERE pu.patient_id = p.id
    "#;
    let stmt = Statement::from_string(manager.get_database_backend(), populate_sql);
    manager.get_connection().execute(stmt).await?;

    // Make user_id NOT NULL now that it's populated
    manager
      .alter_table(
        Table::alter()
          .table(Patients::Table)
          .modify_column(ColumnDef::new(Patients::UserId).integer().not_null())
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx_patients_user_id")
          .table(Patients::Table)
          .col(Patients::UserId)
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_index(Index::drop().name("idx_patients_user_id").table(Patients::Table).to_owned())
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(Patients::Table)
          .drop_foreign_key(Alias::new("fk_patients_user_id"))
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(Table::alter().table(Patients::Table).drop_column(Patients::UserId).to_owned())
      .await?;

    Ok(())
  }
}

#[derive(Iden)]
enum Patients {
  Table,
  UserId,
}

#[derive(Iden)]
enum Users {
  Table,
  Id,
}
