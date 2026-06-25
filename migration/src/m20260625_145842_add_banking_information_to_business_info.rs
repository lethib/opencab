use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum UserBusinessInformations {
  Table,
  BeneficiaryName,
  Iban,
  Bic,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(UserBusinessInformations::Table)
          .add_column(ColumnDef::new(UserBusinessInformations::BeneficiaryName).string())
          .add_column(ColumnDef::new(UserBusinessInformations::Iban).string())
          .add_column(ColumnDef::new(UserBusinessInformations::Bic).string())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(UserBusinessInformations::Table)
          .drop_column(UserBusinessInformations::BeneficiaryName)
          .drop_column(UserBusinessInformations::Iban)
          .drop_column(UserBusinessInformations::Bic)
          .to_owned(),
      )
      .await
  }
}
