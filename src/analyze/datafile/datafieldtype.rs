use std::fmt::Display;
use thiserror::Error;

use crate::analyze::{datafile::DATA_FIELD_MAPPINGS, detection::npi_detection::NPIType};

#[derive(Debug, Clone, PartialEq)]
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
		match self {
			Self::UserIdentity => write!(f, "User Identity"),
			Self::UserRecordNumber => write!(f, "User Record Number"),
			Self::Credential => write!(f, "Credential"),
			Self::SecureCredential => write!(f, "Secure Credential"),
			Self::NPI(npi_type) => write!(f, "Non-Public Information (NPI): {}", npi_type),
			Self::Other => write!(f, "Other/Unknown"),
		}
	}
}

impl DataFieldType {
	pub fn from_field_name(field_name: &str) -> Result<Self, DataFieldTypeError> {
		let field_name_lower = field_name.to_lowercase();

		if let Some(data_field_type) = DATA_FIELD_MAPPINGS.get(field_name_lower.as_str()) {
			Ok(data_field_type.clone())
		} else {
			Ok(DataFieldType::Other)
		}
	}
}

#[derive(Debug, Error)]
pub enum DataFieldTypeError {
	#[error("Data field type not found for the given field name.")]
	FieldTypeNotFound,
}
