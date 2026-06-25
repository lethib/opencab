use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
    // Drop unique constraint on ssn
    m.get_connection()
      .execute_unprepared("ALTER TABLE patients DROP CONSTRAINT patients_ssn_key")
      .await?;

    // Drop unique constraint on hashed_ssn
    m.get_connection()
      .execute_unprepared("ALTER TABLE patients DROP CONSTRAINT patients_hashed_ssn_key")
      .await?;

    Ok(())
  }

  async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
    // Restore unique constraint on ssn
    m.get_connection()
      .execute_unprepared("ALTER TABLE patients ADD CONSTRAINT patients_ssn_key UNIQUE (ssn)")
      .await?;

    // Restore unique constraint on hashed_ssn
    m.get_connection()
      .execute_unprepared("ALTER TABLE patients ADD CONSTRAINT patients_hashed_ssn_key UNIQUE (hashed_ssn)")
      .await?;

    Ok(())
  }
}
