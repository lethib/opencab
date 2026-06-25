use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum MedicalAppointments {
  Table,
  PriceInCents,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(MedicalAppointments::Table)
          .add_column(ColumnDef::new(MedicalAppointments::PriceInCents).integer().null())
          .to_owned(),
      )
      .await?;

    let db = manager.get_connection();
    db.execute_unprepared(
      "UPDATE medical_appointments
       SET price_in_cents = CASE
         WHEN practitioner_office_id = 1 THEN 6000
         WHEN practitioner_office_id = 2 THEN 7500
         ELSE 0
       END",
    )
    .await?;

    manager
      .alter_table(
        Table::alter()
          .table(MedicalAppointments::Table)
          .modify_column(ColumnDef::new(MedicalAppointments::PriceInCents).integer().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(MedicalAppointments::Table)
          .drop_column(MedicalAppointments::PriceInCents)
          .to_owned(),
      )
      .await
  }
}
