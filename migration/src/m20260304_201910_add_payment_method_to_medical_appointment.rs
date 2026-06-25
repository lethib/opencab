use sea_orm_migration::prelude::{extension::postgres::Type, *};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_type(
        Type::create()
          .as_enum(PaymentMethodEnum::Enum)
          .values([
            PaymentMethodEnum::Card,
            PaymentMethodEnum::Cash,
            PaymentMethodEnum::Check,
            PaymentMethodEnum::Transfer,
          ])
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(MedicalAppointments::Table)
          .add_column(
            ColumnDef::new(MedicalAppointments::PaymentMethod)
              .enumeration(
                PaymentMethodEnum::Enum,
                [
                  PaymentMethodEnum::Card,
                  PaymentMethodEnum::Cash,
                  PaymentMethodEnum::Check,
                  PaymentMethodEnum::Transfer,
                ],
              )
              .null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(MedicalAppointments::Table)
          .drop_column(MedicalAppointments::PaymentMethod)
          .to_owned(),
      )
      .await?;

    manager.drop_type(Type::drop().name(PaymentMethodEnum::Enum).to_owned()).await
  }
}

#[derive(Iden)]
enum MedicalAppointments {
  Table,
  PaymentMethod,
}

#[derive(Iden)]
enum PaymentMethodEnum {
  #[iden = "payment_method"]
  Enum,
  #[iden = "card"]
  Card,
  #[iden = "cash"]
  Cash,
  #[iden = "check"]
  Check,
  #[iden = "transfer"]
  Transfer,
}
