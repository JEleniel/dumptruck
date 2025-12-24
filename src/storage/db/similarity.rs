//! Vector similarity search and embedding storage.

use std::io;

use rusqlite::Connection;

/// Compute cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
	if a.len() != b.len() {
		return None;
	}

	let dot_product = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
	let magnitude_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
	let magnitude_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

	if magnitude_a == 0.0 || magnitude_b == 0.0 {
		return None;
	}

	Some(dot_product / (magnitude_a * magnitude_b))
}

/// Update embedding for a canonical address.
pub fn update_address_embedding(
	conn: &Connection,
	canonical_hash: &str,
	embedding: &[f32],
) -> io::Result<()> {
	let embedding_json = serde_json::to_string(embedding)
		.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

	conn.execute(
		"UPDATE canonical_addresses SET embedding = ?1 WHERE canonical_hash = ?2",
		rusqlite::params![&embedding_json, canonical_hash],
	)
	.map_err(io::Error::other)?;

	Ok(())
}

/// Find similar addresses by vector similarity.
pub fn find_similar_addresses(
	conn: &Connection,
	embedding: &[f32],
	limit: usize,
	threshold: f32,
) -> io::Result<Vec<(String, f32)>> {
	let mut stmt = conn
		.prepare(
			"SELECT canonical_hash, embedding FROM canonical_addresses WHERE embedding IS NOT NULL",
		)
		.map_err(io::Error::other)?;

	let rows = stmt
		.query_map([], |row| {
			Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
		})
		.map_err(io::Error::other)?;

	let mut results = Vec::new();
	for row_result in rows {
		let (hash, emb_json) = row_result.map_err(io::Error::other)?;

		if let Ok(stored_emb) = serde_json::from_str::<Vec<f32>>(&emb_json)
			&& let Some(similarity) = cosine_similarity(embedding, &stored_emb)
			&& similarity >= threshold
		{
			results.push((hash, similarity));
		}
	}

	results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
	results.truncate(limit);
	Ok(results)
}

/// Find duplicate by hash or vector similarity.
pub fn find_duplicate_address(
	conn: &Connection,
	canonical_hash: &str,
	embedding: Option<&[f32]>,
	threshold: f32,
) -> io::Result<Option<String>> {
	let mut stmt = conn
		.prepare("SELECT canonical_hash FROM canonical_addresses WHERE canonical_hash = ?1")
		.map_err(io::Error::other)?;

	match stmt.query_row(rusqlite::params![canonical_hash], |_| Ok(())) {
		Ok(()) => return Ok(Some(canonical_hash.to_string())),
		Err(rusqlite::Error::QueryReturnedNoRows) => {}
		Err(e) => return Err(io::Error::other(e)),
	}

	if let Some(emb) = embedding
		&& let Ok(results) = find_similar_addresses(conn, emb, 1, threshold)
		&& let Some((hash, _)) = results.first()
	{
		return Ok(Some(hash.clone()));
	}

	Ok(None)
}
