//! Enrichment plugin trait and a small example implementation.

/// Simple enrichment plugin that derives extra fields from a row.
pub trait EnrichmentPlugin {
	fn enrich(&self, row: &[String]) -> Vec<String>;
}

/// Example enricher that appends a simple checksum field.
pub struct ChecksumEnricher;

impl ChecksumEnricher {
	pub fn new() -> Self {
		ChecksumEnricher
	}
}

impl EnrichmentPlugin for ChecksumEnricher {
	fn enrich(&self, row: &[String]) -> Vec<String> {
		let mut out = row.to_vec();
		let mut sum: u64 = 0;
		for f in row {
			for b in f.as_bytes() {
				sum = sum.wrapping_add(*b as u64);
			}
		}
		out.push(format!("checksum:{}", sum));
		out
	}
}

/// A simple, deterministic enricher used in examples and integration tests.
pub struct SimpleEnricher;

impl SimpleEnricher {
	pub fn new() -> Self {
		SimpleEnricher
	}
}

impl EnrichmentPlugin for SimpleEnricher {
	fn enrich(&self, row: &[String]) -> Vec<String> {
		let mut out = row.to_vec();
		out.push(format!("len:{}", row.len()));
		let mut sum: u64 = 0;
		for f in row {
			for b in f.as_bytes() {
				sum = sum.wrapping_add(*b as u64);
			}
		}
		out.push(format!("checksum:{}", sum));
		out
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn checksum_enricher_appends_field() {
		let enr = ChecksumEnricher::new();
		let row = vec!["alice".to_string(), "bob".to_string()];
		let out = enr.enrich(&row);
		assert_eq!(out.len(), 3);
		assert!(out[2].starts_with("checksum:"));
	}
}
