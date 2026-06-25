use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    // Add unique constraint on user_id to enforce one-to-one relationship
    m.create_index(
      Index::create()
        .name("idx_user_business_informations_user_id_unique")
        .table(Alias::new("user_business_informations"))
        .col(Alias::new("user_id"))
        .unique()
        .to_owned(),
    )
    .await
  }

  async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    // Drop the unique constraint
    m.drop_index(Index::drop().name("idx_user_business_informations_user_id_unique").to_owned())
      .await
  }
}
