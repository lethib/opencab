use sea_orm_migration::{prelude::*, sea_orm::Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum PractitionerOffices {
  Table,
  Id,
  Name,
  #[iden = "address_line_1"]
  AddressLine1,
  AddressZipCode,
  AddressCity,
  AddressCountry,
}

#[derive(Iden)]
enum UserPractitionerOffices {
  Table,
  Id,
  UserId,
  PractitionerOfficeId,
}

#[derive(Iden)]
enum Users {
  Table,
  Id,
}

#[derive(Iden)]
enum Patients {
  Table,
  Office,
}

#[derive(Iden)]
enum PatientUser {
  Table,
  PractitionerOfficeId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Create practitioner_offices table
    manager
      .create_table(
        Table::create()
          .table(PractitionerOffices::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(PractitionerOffices::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(ColumnDef::new(PractitionerOffices::Name).string().not_null())
          .col(ColumnDef::new(PractitionerOffices::AddressLine1).string().not_null())
          .col(ColumnDef::new(PractitionerOffices::AddressZipCode).string().not_null())
          .col(ColumnDef::new(PractitionerOffices::AddressCity).string().not_null())
          .col(ColumnDef::new(PractitionerOffices::AddressCountry).string().not_null())
          .to_owned(),
      )
      .await?;

    // Create user_practitioner_offices join table
    manager
      .create_table(
        Table::create()
          .table(UserPractitionerOffices::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(UserPractitionerOffices::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(ColumnDef::new(UserPractitionerOffices::UserId).integer().not_null())
          .col(
            ColumnDef::new(UserPractitionerOffices::PractitionerOfficeId)
              .integer()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-user_practitioner_offices-user_id")
              .from(UserPractitionerOffices::Table, UserPractitionerOffices::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-user_practitioner_offices-practitioner_office_id")
              .from(UserPractitionerOffices::Table, UserPractitionerOffices::PractitionerOfficeId)
              .to(PractitionerOffices::Table, PractitionerOffices::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    // Remove office column from patients
    manager
      .alter_table(Table::alter().table(Patients::Table).drop_column(Patients::Office).to_owned())
      .await?;

    // Drop the office enum type using raw SQL
    let drop_enum_sql = "DROP TYPE IF EXISTS office";
    let stmt = Statement::from_string(manager.get_database_backend(), drop_enum_sql);
    manager.get_connection().execute_raw(stmt).await?;

    // Add reference from patient_user to practitioner_offices.
    // Uses a DO/EXCEPTION block because the table may have been created as
    // "patient_users" (with 's') in earlier environments — it is dropped entirely
    // in a later migration so this column is transient.
    let sql = r#"
      DO $$ BEGIN
        ALTER TABLE patient_user
          ADD COLUMN practitioner_office_id INTEGER NOT NULL DEFAULT 0;
      EXCEPTION WHEN undefined_table THEN NULL;
      END $$;
    "#;
    let stmt = Statement::from_string(manager.get_database_backend(), sql.to_string());
    manager.get_connection().execute_raw(stmt).await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Remove foreign key and column from patient_user
    manager
      .alter_table(
        Table::alter()
          .table(PatientUser::Table)
          .drop_foreign_key(Alias::new("fk-patient_user-practitioner_office_id"))
          .drop_column(PatientUser::PractitionerOfficeId)
          .to_owned(),
      )
      .await?;

    // Add office column back to patients
    manager
      .alter_table(
        Table::alter()
          .table(Patients::Table)
          .add_column(ColumnDef::new(Patients::Office).string().not_null())
          .to_owned(),
      )
      .await?;

    // Drop tables
    manager
      .drop_table(Table::drop().table(UserPractitionerOffices::Table).to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table(PractitionerOffices::Table).to_owned())
      .await?;

    Ok(())
  }
}
