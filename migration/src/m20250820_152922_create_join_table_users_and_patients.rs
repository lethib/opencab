use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum PatientUsers {
  Table,
  Id,
  UserId,
  PatientId,
}

#[derive(Iden)]
enum Users {
  Table,
  Id,
}

#[derive(Iden)]
enum Patients {
  Table,
  Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(PatientUsers::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(PatientUsers::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(ColumnDef::new(PatientUsers::UserId).integer().not_null())
          .col(ColumnDef::new(PatientUsers::PatientId).integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk-patient_users-user_id")
              .from(PatientUsers::Table, PatientUsers::UserId)
              .to(Users::Table, Users::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-patient_users-patient_id")
              .from(PatientUsers::Table, PatientUsers::PatientId)
              .to(Patients::Table, Patients::Id)
              .on_delete(ForeignKeyAction::Cascade)
              .on_update(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager.drop_table(Table::drop().table(PatientUsers::Table).to_owned()).await
  }
}
