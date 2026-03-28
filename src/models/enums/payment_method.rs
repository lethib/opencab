use crate::models::_entities::sea_orm_active_enums::PaymentMethod;

impl PaymentMethod {
  pub fn to_french(&self) -> &str {
    match self {
      PaymentMethod::Card => "Carte Bancaire",
      PaymentMethod::Cash => "Espèces",
      PaymentMethod::Check => "Chèque",
      PaymentMethod::Transfer => "Virement",
    }
  }
}
