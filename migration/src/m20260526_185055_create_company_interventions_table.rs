use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table("company_interventions")
          .if_not_exists()
          .col(integer("id").auto_increment().primary_key())
          .col(integer("company_id"))
          .col(integer("practitioner_id"))
          .col(integer("quantity"))
          .col(integer("unit_price_in_cents"))
          .col(decimal_len("vat_rate_in_percent", 4, 1))
          .col(date("issue_date"))
          .col(string("object"))
          .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
          .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
          .foreign_key(
            ForeignKey::create()
              .name("fk-company_interventions-company_id")
              .from_col("company_id")
              .to("practitioner_companies", "id")
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-company_interventions-practitioner_id")
              .from_col("practitioner_id")
              .to("users", "id")
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx-company_interventions-company_id")
          .table("company_interventions")
          .col("company_id")
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx-company_interventions-practitioner_id")
          .table("company_interventions")
          .col("practitioner_id")
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_index(Index::drop().name("idx-company_interventions-practitioner_id").to_owned())
      .await?;

    manager
      .drop_index(Index::drop().name("idx-company_interventions-company_id").to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table("company_interventions").to_owned())
      .await
  }
}
