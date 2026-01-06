//! Detection and normalization of Personally Identifiable Information (PII) and Non-Public
//! Information (NPI).
mod accountnumber;
mod bankiban;
mod bankroutingnumber;
mod bankswiftcode;
mod biometricdata;
mod creditcardnumber;
mod cryptoaddress;
mod dateofbirth;
mod emailaddress;
mod genderdata;
mod gpslocation;
mod imeinumber;
mod isocountrycodes;
mod mailingaddress;
mod nationalidentificationnumber;
mod normalizer;
mod npitype;
mod otheridentificationnumber;
mod otheridentity;
mod otherpersonaldata;
mod otherrecordnumber;
mod personalidentificationnumber;
mod personalname;
mod phonenumber;

pub use npitype::*;

use std::collections::HashMap;

use crate::analyze::datafile::DataFieldType;

#[derive(Debug, Clone)]
pub struct NPIResult {
	pub npi_detected: bool,
	pub npi_type: Option<NPIType>,
	pub confidence: f32,
}

/// Analyze a field value and detect any PII/NPI
pub fn detect_npi(value: &str, column_data_type: DataFieldType) -> NPIResult {
	let mut detected: HashMap<NPIType, f32> = HashMap::new();

	let trimmed = value.trim();

	detected.insert(
		NPIType::AccountNumber,
		accountnumber::AccountNumber::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::BankIBAN,
		bankiban::BankIBAN::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::BankRoutingNumber,
		bankroutingnumber::BankRoutingNumber::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::BankSWIFTCode,
		bankswiftcode::BankSWIFTCode::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::BiometricData,
		biometricdata::BiometricData::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::CreditCardNumber,
		creditcardnumber::CreditCardNumber::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::CryptoAddress,
		cryptoaddress::CryptoAddress::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::DateOfBirth,
		dateofbirth::DateOfBirth::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::EmailAddress,
		emailaddress::EmailAddress::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::PersonalName,
		personalname::PersonalName::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::GenderData,
		genderdata::GenderData::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::GPSLocation,
		gpslocation::GPSLocation::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::IMEI,
		imeinumber::IMEINumber::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::MailingAddress,
		mailingaddress::MailingAddress::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::NationalIdentificationNumber,
		nationalidentificationnumber::NationalIdentificationNumber::detect(
			trimmed,
			column_data_type,
		)
		.unwrap_or(0.0),
	);
	detected.insert(
		NPIType::PersonalIdentificationNumber,
		personalidentificationnumber::PersonalIdentificationNumber::detect(
			trimmed,
			column_data_type,
		)
		.unwrap_or(0.0),
	);
	detected.insert(
		NPIType::PhoneNumber,
		phonenumber::PhoneNumber::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::OtherIdentificationNumber(""),
		otheridentificationnumber::OtherIdentificationNumber::detect(trimmed, column_data_type)
			.unwrap_or(0.0),
	);
	detected.insert(
		NPIType::OtherIdentity(""),
		otheridentity::OtherIdentity::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::OtherPersonalData(""),
		otherpersonaldata::OtherPersonalData::detect(trimmed, column_data_type).unwrap_or(0.0),
	);
	detected.insert(
		NPIType::OtherRecordNumber(""),
		otherrecordnumber::OtherRecordNumber::detect(trimmed, column_data_type).unwrap_or(0.0),
	);

	let result: NPIResult = if let Some((npi_type, &confidence)) =
		detected.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
	{
		if confidence >= 0.8 {
			NPIResult {
				npi_detected: true,
				npi_type: Some(npi_type.clone()),
				confidence,
			}
		} else {
			NPIResult {
				npi_detected: false,
				npi_type: None,
				confidence: 0.0,
			}
		}
	} else {
		NPIResult {
			npi_detected: false,
			npi_type: None,
			confidence: 0.0,
		}
	};
	result
}
