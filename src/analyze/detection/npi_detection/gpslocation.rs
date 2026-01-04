use regex::Regex;

use crate::{datafile::DataFieldType, detection::DetectionError};

pub struct GPSLocation {}

impl GPSLocation {
	pub fn detect(value: &str, column_type: DataFieldType) -> Result<f32, DetectionError> {
		// GPS coordinates: decimal degrees or DMS format
		let mut confidence: f32 = 0.0;

		if column_type == DataFieldType::NPI {
			confidence += 0.2;
		}

		// Comma-separated lat,lon format
		if Self::is_lat_lon_pair(value) {
			return Ok(confidence + 0.8);
		}

		// Decimal degrees format: -90 to 90 for latitude, -180 to 180 for longitude
		if Self::is_decimal_degrees(value) {
			return Ok(confidence + 0.7);
		}

		// DMS (Degrees, Minutes, Seconds) format: 40:26:46N, 120:15:30W
		if Self::is_dms_format(value) {
			return Ok(confidence + 0.7);
		}

		Ok(confidence)
	}

	fn is_decimal_degrees(value: &str) -> bool {
		let regex =
			Regex::new(r"^-?([0-9]{1,2}|90)\.?[0-9]*\s*,\s*-?([0-9]{1,3}|180)\.?[0-9]*$").ok();
		if let Some(re) = regex {
			return re.is_match(value);
		}
		false
	}

	fn is_dms_format(value: &str) -> bool {
		// DMS format: 40:26:46N or 40:26:46, with support for variations
		let regex = Regex::new(
			r"^\d{1,2}:\d{1,2}:\d{1,2}[NSEWnsew]?\s*,?\s*\d{1,3}:\d{1,2}:\d{1,2}[NSEWnsew]?$",
		)
		.ok();
		if let Some(re) = regex {
			return re.is_match(value);
		}
		false
	}

	fn is_lat_lon_pair(value: &str) -> bool {
		let parts: Vec<&str> = value.split(',').collect();
		if parts.len() != 2 {
			return false;
		}

		parts.iter().all(|part| {
			part.trim()
				.parse::<f64>()
				.map(|f| (f >= -90.0 && f <= 90.0) || (f >= -180.0 && f <= 180.0))
				.unwrap_or(false)
		})
	}
}
