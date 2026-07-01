use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.create_index(
      Index::create()
        .name("idx_users_pid_unique")
        .table("users")
        .col("pid")
        .unique()
        .to_owned(),
    )
    .await
  }

  async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    m.drop_index(Index::drop().name("idx_users_pid_unique").to_owned()).await
  }
}
