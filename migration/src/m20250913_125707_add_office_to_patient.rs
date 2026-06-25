use sea_orm_migration::{
  prelude::*,
  sea_orm::{EnumIter, Iterable, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Create the office enum type using raw SQL
    let create_enum_sql = "CREATE TYPE office AS ENUM ('Vitry-sur-Seine', 'Rueil-Malmaison')";
    let stmt = Statement::from_string(manager.get_database_backend(), create_enum_sql);
    manager.get_connection().execute(stmt).await?;

    // Add the office column to the patients table
    manager
      .alter_table(
        Table::alter()
          .table(Patients::Table)
          .add_column(
            ColumnDef::new(Patients::Office)
              .enumeration(Alias::new("office"), Office::iter())
              .not_null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Drop the office column from the patients table
    manager
      .alter_table(Table::alter().table(Patients::Table).drop_column(Patients::Office).to_owned())
      .await?;

    // Drop the office enum type using raw SQL
    let drop_enum_sql = "DROP TYPE IF EXISTS office";
    let stmt = Statement::from_string(manager.get_database_backend(), drop_enum_sql);
    manager.get_connection().execute(stmt).await?;

    Ok(())
  }
}

#[derive(Iden, EnumIter)]
pub enum Office {
  #[iden = "Vitry-sur-Seine"]
  VitrySurSeine,
  #[iden = "Rueil-Malmaison"]
  RueilMalmaison,
}

#[derive(Iden)]
enum Patients {
  Table,
  Office,
}
