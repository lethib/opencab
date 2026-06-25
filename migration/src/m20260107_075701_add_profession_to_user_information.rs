use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum UserBusinessInformations {
  Table,
  Profession,
}

#[derive(Iden)]
enum ProfessionEnum {
  #[iden = "profession"]
  Enum,
  // Medical professions
  #[iden = "general_practitioner"]
  GeneralPractitioner,
  #[iden = "pediatrician"]
  Pediatrician,
  #[iden = "gynecologist"]
  Gynecologist,
  #[iden = "psychiatrist"]
  Psychiatrist,
  #[iden = "gastroenterologist"]
  Gastroenterologist,
  #[iden = "ent_specialist"]
  ENTSpecialist,
  #[iden = "endocrinologist"]
  Endocrinologist,
  #[iden = "cardiologist"]
  Cardiologist,
  #[iden = "angiologist"]
  Angiologist,
  #[iden = "nephrologist"]
  Nephrologist,
  #[iden = "neurologist"]
  Neurologist,
  #[iden = "pulmonologist"]
  Pulmonologist,
  #[iden = "rheumatologist"]
  Rheumatologist,
  #[iden = "dermatologist"]
  Dermatologist,
  #[iden = "dentist"]
  Dentist,
  #[iden = "midwife"]
  Midwife,
  // Paramedical professions
  #[iden = "physiotherapist"]
  Physiotherapist,
  #[iden = "nurse"]
  Nurse,
  #[iden = "psychologist"]
  Psychologist,
  #[iden = "osteopath"]
  Osteopath,
  // Other regulated health professions
  #[iden = "audiologist"]
  Audiologist,
  #[iden = "chiropractor"]
  Chiropractor,
  #[iden = "genetic_counselor"]
  GeneticCounselor,
  #[iden = "dietitian"]
  Dietitian,
  #[iden = "occupational_therapist"]
  OccupationalTherapist,
  #[iden = "speech_therapist"]
  SpeechTherapist,
  #[iden = "orthoptist"]
  Orthoptist,
  #[iden = "podiatrist"]
  Podiatrist,
  #[iden = "psychomotrician"]
  Psychomotrician,
  #[iden = "psychotherapist"]
  Psychotherapist,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Create the profession enum type
    manager
      .create_type(
        Type::create()
          .as_enum(ProfessionEnum::Enum)
          .values([
            ProfessionEnum::GeneralPractitioner,
            ProfessionEnum::Pediatrician,
            ProfessionEnum::Gynecologist,
            ProfessionEnum::Psychiatrist,
            ProfessionEnum::Gastroenterologist,
            ProfessionEnum::ENTSpecialist,
            ProfessionEnum::Endocrinologist,
            ProfessionEnum::Cardiologist,
            ProfessionEnum::Angiologist,
            ProfessionEnum::Nephrologist,
            ProfessionEnum::Neurologist,
            ProfessionEnum::Pulmonologist,
            ProfessionEnum::Rheumatologist,
            ProfessionEnum::Dermatologist,
            ProfessionEnum::Dentist,
            ProfessionEnum::Midwife,
            ProfessionEnum::Physiotherapist,
            ProfessionEnum::Nurse,
            ProfessionEnum::Psychologist,
            ProfessionEnum::Osteopath,
            ProfessionEnum::Audiologist,
            ProfessionEnum::Chiropractor,
            ProfessionEnum::GeneticCounselor,
            ProfessionEnum::Dietitian,
            ProfessionEnum::OccupationalTherapist,
            ProfessionEnum::SpeechTherapist,
            ProfessionEnum::Orthoptist,
            ProfessionEnum::Podiatrist,
            ProfessionEnum::Psychomotrician,
            ProfessionEnum::Psychotherapist,
          ])
          .to_owned(),
      )
      .await?;

    // Add the profession column to the table as nullable first
    manager
      .alter_table(
        Table::alter()
          .table(UserBusinessInformations::Table)
          .add_column(
            ColumnDef::new(UserBusinessInformations::Profession)
              .enumeration(
                ProfessionEnum::Enum,
                [
                  ProfessionEnum::GeneralPractitioner,
                  ProfessionEnum::Pediatrician,
                  ProfessionEnum::Gynecologist,
                  ProfessionEnum::Psychiatrist,
                  ProfessionEnum::Gastroenterologist,
                  ProfessionEnum::ENTSpecialist,
                  ProfessionEnum::Endocrinologist,
                  ProfessionEnum::Cardiologist,
                  ProfessionEnum::Angiologist,
                  ProfessionEnum::Nephrologist,
                  ProfessionEnum::Neurologist,
                  ProfessionEnum::Pulmonologist,
                  ProfessionEnum::Rheumatologist,
                  ProfessionEnum::Dermatologist,
                  ProfessionEnum::Dentist,
                  ProfessionEnum::Midwife,
                  ProfessionEnum::Physiotherapist,
                  ProfessionEnum::Nurse,
                  ProfessionEnum::Psychologist,
                  ProfessionEnum::Osteopath,
                  ProfessionEnum::Audiologist,
                  ProfessionEnum::Chiropractor,
                  ProfessionEnum::GeneticCounselor,
                  ProfessionEnum::Dietitian,
                  ProfessionEnum::OccupationalTherapist,
                  ProfessionEnum::SpeechTherapist,
                  ProfessionEnum::Orthoptist,
                  ProfessionEnum::Podiatrist,
                  ProfessionEnum::Psychomotrician,
                  ProfessionEnum::Psychotherapist,
                ],
              )
              .null(),
          )
          .to_owned(),
      )
      .await?;

    // Set default value for existing records
    let db = manager.get_connection();
    db.execute_unprepared(
      "UPDATE user_business_informations SET profession = 'general_practitioner'::profession WHERE profession IS NULL",
    )
    .await?;

    // Make the column NOT NULL
    manager
      .alter_table(
        Table::alter()
          .table(UserBusinessInformations::Table)
          .modify_column(
            ColumnDef::new(UserBusinessInformations::Profession)
              .enumeration(
                ProfessionEnum::Enum,
                [
                  ProfessionEnum::GeneralPractitioner,
                  ProfessionEnum::Pediatrician,
                  ProfessionEnum::Gynecologist,
                  ProfessionEnum::Psychiatrist,
                  ProfessionEnum::Gastroenterologist,
                  ProfessionEnum::ENTSpecialist,
                  ProfessionEnum::Endocrinologist,
                  ProfessionEnum::Cardiologist,
                  ProfessionEnum::Angiologist,
                  ProfessionEnum::Nephrologist,
                  ProfessionEnum::Neurologist,
                  ProfessionEnum::Pulmonologist,
                  ProfessionEnum::Rheumatologist,
                  ProfessionEnum::Dermatologist,
                  ProfessionEnum::Dentist,
                  ProfessionEnum::Midwife,
                  ProfessionEnum::Physiotherapist,
                  ProfessionEnum::Nurse,
                  ProfessionEnum::Psychologist,
                  ProfessionEnum::Osteopath,
                  ProfessionEnum::Audiologist,
                  ProfessionEnum::Chiropractor,
                  ProfessionEnum::GeneticCounselor,
                  ProfessionEnum::Dietitian,
                  ProfessionEnum::OccupationalTherapist,
                  ProfessionEnum::SpeechTherapist,
                  ProfessionEnum::Orthoptist,
                  ProfessionEnum::Podiatrist,
                  ProfessionEnum::Psychomotrician,
                  ProfessionEnum::Psychotherapist,
                ],
              )
              .not_null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Drop the profession column
    manager
      .alter_table(
        Table::alter()
          .table(UserBusinessInformations::Table)
          .drop_column(UserBusinessInformations::Profession)
          .to_owned(),
      )
      .await?;

    // Drop the profession enum type
    manager.drop_type(Type::drop().name(ProfessionEnum::Enum).to_owned()).await
  }
}
