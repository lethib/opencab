use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table("practitioner_companies")
          .if_not_exists()
          .col(integer("id").auto_increment().primary_key())
          .col(string("name"))
          .col(string("contact_email"))
          .col(string_null("address_line_1"))
          .col(string_null("address_zip_code"))
          .col(string_null("address_country"))
          .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
          .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
          .col(integer("user_id"))
          .foreign_key(
            ForeignKey::create()
              .name("fk-companies-user_id")
              .from_col("user_id")
              .to("users", "id")
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await?;

    manager
      .create_index(
        Index::create()
          .name("idx-companies-user_id")
          .table("practitioner_companies")
          .col("user_id")
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_index(Index::drop().name("idx-companies-user_id").to_owned())
      .await?;

    manager
      .drop_table(Table::drop().table("practitioner_companies").to_owned())
      .await
  }
}
