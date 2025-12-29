/// Input file format specification
#[derive(Debug, Clone)]
pub enum DataFileType {
	/// Comma-separated values
	Csv,
	/// Pipe-separated values
	Psv,
	/// Tab-separated values
	Tsv,
}
