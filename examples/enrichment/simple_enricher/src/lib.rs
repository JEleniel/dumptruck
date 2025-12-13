//! A minimal Rust enrichment example for Dumptruck.
//!
//! This crate demonstrates a simple enricher that appends a `len:` tag
//! and a lightweight checksum. It's intended as a reference for how to
//! implement the enrichment logic in Rust. To integrate with the main
//! `dumptruck` crate, implement the `EnrichmentPlugin` trait from
//! `src/enrichment.rs` and wire the plugin into the pipeline.

pub struct SimpleEnricher;

impl SimpleEnricher {
    pub fn new() -> Self {
        SimpleEnricher
    }

    /// Enrich a normalized row by appending derived fields.
    pub fn enrich(&self, row: &[String]) -> Vec<String> {
        let mut out = row.to_vec();
        // append a `len:` tag with number of fields
        out.push(format!("len:{}", row.len()));
        // append a small additive checksum for quick inspection
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
    fn simple_enricher_appends_fields() {
        let e = SimpleEnricher::new();
        let row = vec!["alice".to_string(), "bob".to_string()];
        let out = e.enrich(&row);
        assert_eq!(out.len(), 4);
        assert!(out.iter().any(|s| s.starts_with("len:")));
        assert!(out.iter().any(|s| s.starts_with("checksum:")));
    }
}
