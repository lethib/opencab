mod factories;
mod steps;

use cucumber::World;
use migration::{Migrator, MigratorTrait};
use opencab::{
  models::{
    medical_appointments::Model as AppointmentModel, my_errors::MyErrors,
    patients::Model as PatientModel, practitioner_offices::Model as OfficeModel,
    users::Model as UserModel,
  },
  services::appointments::MedicalAppointmentDetail,
};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};

const DEFAULT_TEST_DATABASE_URL: &str = "postgres://loco:loco@localhost:5431/opencab_test";

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct AppWorld {
  pub db: DatabaseConnection,
  pub crypto: CryptoState,
  pub appointments: AppointmentsState,
  pub practitioner_office: PractitionerOfficeState,
}

impl AppWorld {
  async fn new() -> Self {
    let db_url =
      std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());

    let db = Database::connect(&db_url).await.unwrap();

    // Reset all data between scenarios — equivalent of RSpec's database_cleaner
    db.execute_unprepared(
      "TRUNCATE TABLE medical_appointments, user_practitioner_offices,
             user_business_informations, patients, practitioner_offices, users
             RESTART IDENTITY CASCADE",
    )
    .await
    .unwrap();

    Self {
      db,
      crypto: CryptoState::default(),
      appointments: AppointmentsState::default(),
      practitioner_office: PractitionerOfficeState::default(),
    }
  }
}

#[derive(Debug, Default)]
pub struct CryptoState {
  pub encrypted: Option<String>,
  pub second_encrypted: Option<String>,
  pub hashed: Option<String>,
  pub second_hashed: Option<String>,
  pub decrypt_failed: bool,
}

#[derive(Debug, Default)]
pub struct PractitionerOfficeState {
  pub user: Option<UserModel>,
  pub office: Option<OfficeModel>,
  pub last_error: Option<MyErrors>,
}

#[derive(Debug, Default)]
pub struct AppointmentsState {
  pub user: Option<UserModel>,
  pub patient: Option<PatientModel>,
  pub office: Option<OfficeModel>,
  pub second_office: Option<OfficeModel>,
  pub appointment: Option<AppointmentModel>,
  pub extracted: Vec<MedicalAppointmentDetail>,
}

impl AppointmentsState {
  pub fn all_offices(&self) -> impl Iterator<Item = &OfficeModel> {
    self.office.iter().chain(self.second_office.iter())
  }
}

#[tokio::main]
async fn main() {
  std::env::set_var("SSN_ENCRYPTION_KEY", "12345678901234567890123456789012");
  std::env::set_var("SSN_SALT_KEY", "bdd_test_salt_key_for_patients!!");

  let db_url =
    std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
  let db = Database::connect(&db_url).await.unwrap();
  Migrator::up(&db, None).await.unwrap();

  AppWorld::cucumber()
    .max_concurrent_scenarios(1)
    .run("tests/features")
    .await;
}
