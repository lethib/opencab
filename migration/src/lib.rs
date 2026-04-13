#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::wildcard_imports)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_users;

mod m20250809_080459_remove_unnecessary_cols_from_users;
mod m20250820_151249_patients;
mod m20250820_152922_create_join_table_users_and_patients;
mod m20250902_193546_add_hashed_ssn_to_patients;
mod m20250907_000001_split_patient_name;
mod m20250910_203200_add_address_to_patients;
mod m20250912_161133_user_business_informations;
mod m20250912_195210_add_unique_constraint_user_business_info;
mod m20250913_125707_add_office_to_patient;
mod m20250921_200853_add_phone_number_to_user;
mod m20250921_201555_change_user_name_to_first_and_last_name;
mod m20250922_074830_create_practitioner_office_table;
mod m20250930_145445_add_signature_file_name_to_user_business_information;
mod m20251018_123530_add_email_to_patient;
mod m20251025_210751_remove_unique_constraints_from_patient_ssn;
mod m20251122_092542_add_user_id_to_patients;
mod m20251123_103746_create_medicale_appointments_table;
mod m20251129_135113_add_default_timestamps_to_medical_appointments;
mod m20251201_220702_drop_patient_user_table;
mod m20251208_221001_add_access_key_to_user;
mod m20251220_215546_make_signature_filename_nullable;
mod m20260107_075701_add_profession_to_user_information;
mod m20260113_223518_add_pricing_to_medical_appointments;
mod m20260304_201910_add_payment_method_to_medical_appointment;
mod m20260308_000001_fix_schema_drift;
mod m20260310_175025_add_revenue_share_percentage_to_user_practitioner_office;
mod m20260317_220127_make_patient_email_nullable;
mod m20260412_074522_make_patient_ssn_nullable;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
      Box::new(m20220101_000001_users::Migration),
      Box::new(m20250809_080459_remove_unnecessary_cols_from_users::Migration),
      Box::new(m20250820_151249_patients::Migration),
      Box::new(m20250820_152922_create_join_table_users_and_patients::Migration),
      Box::new(m20250902_193546_add_hashed_ssn_to_patients::Migration),
      Box::new(m20250907_000001_split_patient_name::Migration),
      Box::new(m20250910_203200_add_address_to_patients::Migration),
      Box::new(m20250912_161133_user_business_informations::Migration),
      Box::new(m20250912_195210_add_unique_constraint_user_business_info::Migration),
      Box::new(m20250913_125707_add_office_to_patient::Migration),
      Box::new(m20250921_200853_add_phone_number_to_user::Migration),
      Box::new(m20250921_201555_change_user_name_to_first_and_last_name::Migration),
      Box::new(m20250922_074830_create_practitioner_office_table::Migration),
      Box::new(m20250930_145445_add_signature_file_name_to_user_business_information::Migration),
      Box::new(m20251018_123530_add_email_to_patient::Migration),
      Box::new(m20251025_210751_remove_unique_constraints_from_patient_ssn::Migration),
      Box::new(m20251122_092542_add_user_id_to_patients::Migration),
      Box::new(m20251123_103746_create_medicale_appointments_table::Migration),
      Box::new(m20251129_135113_add_default_timestamps_to_medical_appointments::Migration),
      Box::new(m20251201_220702_drop_patient_user_table::Migration),
      Box::new(m20251208_221001_add_access_key_to_user::Migration),
      Box::new(m20251220_215546_make_signature_filename_nullable::Migration),
      Box::new(m20260107_075701_add_profession_to_user_information::Migration),
      Box::new(m20260113_223518_add_pricing_to_medical_appointments::Migration),
      Box::new(m20260304_201910_add_payment_method_to_medical_appointment::Migration),
      Box::new(m20260308_000001_fix_schema_drift::Migration),
      Box::new(
        m20260310_175025_add_revenue_share_percentage_to_user_practitioner_office::Migration,
      ),
      Box::new(m20260317_220127_make_patient_email_nullable::Migration),
      Box::new(m20260412_074522_make_patient_ssn_nullable::Migration),
    ]
  }
}
