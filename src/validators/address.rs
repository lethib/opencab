use once_cell::sync::Lazy;
use regex::Regex;

/// French postal code validation regex
static FR_ZIP_CODE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?:0[1-9]|[1-8]\d|9[0-8])\d{3}$").unwrap());

fn validate_postal_code(zip_code: &str) -> bool {
  !zip_code.is_empty() && FR_ZIP_CODE_REGEX.is_match(zip_code)
}

fn validate_address_line_length(address_line: &str) -> bool {
  address_line.len() < 100
}

pub fn is_address_valid(address_line_1: &str, zip_code: &str) -> bool {
  if !validate_address_line_length(address_line_1) {
    return false;
  }

  if !validate_postal_code(zip_code) {
    return false;
  }

  true
}
