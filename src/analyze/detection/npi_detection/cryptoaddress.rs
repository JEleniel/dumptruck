use regex::Regex;

use crate::analyze::{
	datafile::DataFieldType,
	detection::{DetectionError, npi_detection::NPIType},
};

pub struct CryptoAddress {}

impl CryptoAddress {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// Crypto wallet addresses (Bitcoin, Ethereum, etc.)
		// Identified best by checksums:
		// BTC / LTC / XRP / TRX / SOL / DOT: Base58Check checksum
		// ETH: EIP-55 mixed-case checksum
		// Bech32 (bc1, ltc1, addr1): Bech32 checksum
		// XMR: Network byte + checksum
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI(NPIType::CryptoAddress) {
			confidence += 0.5;
		}

		// Bitcoin addresses (P2PKH: 1..., P2SH: 3..., Bech32: bc1...)
		if Self::validate_bitcoin(value) {
			return Ok(confidence + 0.5);
		}

		// Ethereum addresses (0x...)
		if Self::validate_ethereum(value) {
			return Ok(confidence + 0.5);
		}

		// Bech32 format (generic)
		if Self::validate_bech32(value) {
			return Ok(confidence + 0.4);
		}

		// Base58 format (generic)
		if Self::validate_base58(value) {
			return Ok(confidence + 0.3);
		}

		Ok(confidence)
	}

	fn validate_bitcoin(value: &str) -> bool {
		// Bitcoin address: starts with 1, 3, or bc1
		let regex = Regex::new(r"^(1|3)[1-9A-HJ-NP-Z]{24,33}$").ok();
		if let Some(re) = regex {
			if re.is_match(value) {
				return true;
			}
		}

		// Bech32 Bitcoin (bc1...)
		let bech32_regex = Regex::new(r"^bc1[a-z0-9]{39,59}$").ok();
		if let Some(re) = bech32_regex {
			return re.is_match(value);
		}

		false
	}

	fn validate_ethereum(value: &str) -> bool {
		let regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn validate_bech32(value: &str) -> bool {
		let regex = Regex::new(r"^[a-z]{2,6}1[a-z0-9]{39,59}$").ok();
		regex.is_some_and(|re| re.is_match(value))
	}

	fn validate_base58(value: &str) -> bool {
		let regex = Regex::new(r"^[1-9A-HJ-NP-Z]{26,35}$").ok();
		regex.is_some_and(|re| re.is_match(value))
	}
}
