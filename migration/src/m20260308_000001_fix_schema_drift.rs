use sea_orm_migration::{prelude::*, sea_orm::Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Fixes schema drift between the dev DB (created before migrations tracked these details)
/// and fresh DBs created from scratch via migrations.
///
/// Safe to run on any environment:
/// - Dev DB: columns already exist / names already correct → all operations are no-ops
/// - Fresh DB: applies all needed changes
#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    let db = manager.get_database_backend();

    let stmts = [
      // Add timestamps to tables missing them
      "ALTER TABLE users ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()",
      "ALTER TABLE patients ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()",
      "ALTER TABLE practitioner_offices ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()",
      "ALTER TABLE user_business_informations ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()",
      "ALTER TABLE user_practitioner_offices ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()",
      // Rename address_line1 -> address_line_1 if the old name exists
      "DO $$ BEGIN ALTER TABLE patients RENAME COLUMN address_line1 TO address_line_1; EXCEPTION WHEN undefined_column THEN NULL; END $$",
      "DO $$ BEGIN ALTER TABLE practitioner_offices RENAME COLUMN address_line1 TO address_line_1; EXCEPTION WHEN undefined_column THEN NULL; END $$",
    ];

    for sql in stmts {
      conn.execute_raw(Statement::from_string(db, sql)).await?;
    }

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let conn = manager.get_connection();
    let db = manager.get_database_backend();

    let stmts = [
      "DO $$ BEGIN ALTER TABLE patients RENAME COLUMN address_line_1 TO address_line1; EXCEPTION WHEN undefined_column THEN NULL; END $$",
      "DO $$ BEGIN ALTER TABLE practitioner_offices RENAME COLUMN address_line_1 TO address_line1; EXCEPTION WHEN undefined_column THEN NULL; END $$",
      "ALTER TABLE users DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at",
      "ALTER TABLE patients DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at",
      "ALTER TABLE practitioner_offices DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at",
      "ALTER TABLE user_business_informations DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at",
      "ALTER TABLE user_practitioner_offices DROP COLUMN IF EXISTS created_at, DROP COLUMN IF EXISTS updated_at",
    ];

    for sql in stmts {
      conn.execute_raw(Statement::from_string(db, sql)).await?;
    }

    Ok(())
  }
}
