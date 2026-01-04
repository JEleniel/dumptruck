use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;

use crate::analyze::{datafile::DATA_FIELD_MAPPINGS, detection::npi_detection::NPIType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFieldType {
	UserIdentity,
	UserRecordNumber,
	Credential,
	SecureCredential,
	NPI(NPIType),
	Other,
}

impl Display for DataFieldType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let text = match self {
			Self::UserIdentity => "User Identity",
			Self::UserRecordNumber => "User Record Number",
			Self::Credential => "Credential",
			Self::SecureCredential => "Secure Credential",
			Self::NPI(npi_type) => format!("Non-Public Information (NPI): {}", npi_type).as_str(),
			Self::Other => "Other/Unknown",
		};
		write!(f, "{}", text)
	}
}

impl DataFieldType {
	pub fn from_field_name(field_name: &str) -> Result<Self, DataFieldTypeError> {
		let field_name_lower = field_name.to_lowercase();

		if DATA_FIELD_MAPPINGS.contains_key(&field_name_lower) {
			Ok(DATA_FIELD_MAPPINGS.get(&field_name_lower)?.clone())
		} else {
			Ok(DataFieldType::Other)
		}
	}
}

#[derive(Debug, Error)]
pub enum DataFieldTypeError {}
