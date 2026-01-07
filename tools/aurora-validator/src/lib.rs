use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AuroraCard {
	pub id: String,
	#[serde(rename = "type")]
	pub card_type: String,
	pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuroraLink {
	pub id: String,
	pub source: String,
	pub target: String,
}

pub fn validate_repo_default() -> Result<(), String> {
	validate_paths(
		Path::new("docs/design/AURORA/cards"),
		Path::new("docs/design/AURORA/links"),
		"driver:dumptruck-root",
	)
}

pub fn validate_paths(cards_dir: &Path, links_dir: &Path, root_id: &str) -> Result<(), String> {
	let cards = load_cards_from_dir(cards_dir)?;
	let links = load_links_from_dir(links_dir)?;
	validate_model(&cards, &links, root_id)
}

pub fn load_cards_from_dir(dir: &Path) -> Result<HashMap<String, AuroraCard>, String> {
	let mut entries = fs::read_dir(dir)
		.map_err(|err| format!("failed to read dir {dir:?}: {err}"))?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|err| format!("failed to read entries in {dir:?}: {err}"))?;
	entries.sort_by_key(|entry| entry.file_name());

	let mut cards = HashMap::new();
	for entry in entries {
		let path = entry.path();
		if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
			continue;
		}

		let raw =
			fs::read_to_string(&path).map_err(|err| format!("failed to read {path:?}: {err}"))?;
		let card: AuroraCard = serde_json::from_str(&raw)
			.map_err(|err| format!("failed to parse {path:?} as AuroraCard: {err}"))?;

		if cards.insert(card.id.clone(), card).is_some() {
			return Err(format!("duplicate card id found in {dir:?}: {path:?}"));
		}
	}

	if cards.is_empty() {
		return Err(format!("no AURORA card JSON files found in {dir:?}"));
	}

	Ok(cards)
}

pub fn load_links_from_dir(dir: &Path) -> Result<Vec<AuroraLink>, String> {
	let mut entries = fs::read_dir(dir)
		.map_err(|err| format!("failed to read dir {dir:?}: {err}"))?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|err| format!("failed to read entries in {dir:?}: {err}"))?;
	entries.sort_by_key(|entry| entry.file_name());

	let mut links = Vec::new();
	for entry in entries {
		let path = entry.path();
		if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
			continue;
		}

		let raw =
			fs::read_to_string(&path).map_err(|err| format!("failed to read {path:?}: {err}"))?;
		let link: AuroraLink = serde_json::from_str(&raw)
			.map_err(|err| format!("failed to parse {path:?} as AuroraLink: {err}"))?;

		links.push(link);
	}

	if links.is_empty() {
		return Err(format!("no AURORA link JSON files found in {dir:?}"));
	}

	Ok(links)
}

pub fn validate_model(
	cards: &HashMap<String, AuroraCard>,
	links: &[AuroraLink],
	root_id: &str,
) -> Result<(), String> {
	let root = cards
		.get(root_id)
		.ok_or_else(|| format!("root driver card is missing: {root_id}"))?;
	if root.card_type != "driver" {
		return Err(format!(
			"root card {root_id} must have type=driver, got {}",
			root.card_type
		));
	}

	let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
	for link in links {
		if !cards.contains_key(&link.source) {
			return Err(format!(
				"link {} references missing source card: {}",
				link.id, link.source
			));
		}
		if !cards.contains_key(&link.target) {
			return Err(format!(
				"link {} references missing target card: {}",
				link.id, link.target
			));
		}
		if link.source == link.target {
			return Err(format!(
				"link {} has identical source and target: {}",
				link.id, link.source
			));
		}
		outgoing
			.entry(link.source.clone())
			.or_default()
			.push(link.target.clone());
	}

	if outgoing
		.get(root_id)
		.map_or(false, |targets| !targets.is_empty())
	{
		return Err(format!("root card {root_id} must not have outgoing links"));
	}

	for card_id in cards.keys() {
		if card_id == root_id {
			continue;
		}
		if match outgoing.get(card_id) {
			Some(targets) => targets.is_empty(),
			None => true,
		} {
			return Err(format!(
				"non-root card must have at least one outgoing link: {card_id}"
			));
		}
	}

	let mut visiting = HashSet::<String>::new();
	let mut visited = HashSet::<String>::new();
	for card_id in cards.keys() {
		visit_for_cycles(card_id, &outgoing, &mut visiting, &mut visited)?;
	}

	for card_id in cards.keys() {
		if card_id == root_id {
			continue;
		}
		if !can_reach_root(card_id, root_id, &outgoing) {
			return Err(format!("card does not reach root driver: {card_id}"));
		}
	}

	Ok(())
}

fn visit_for_cycles(
	card_id: &str,
	outgoing: &HashMap<String, Vec<String>>,
	visiting: &mut HashSet<String>,
	visited: &mut HashSet<String>,
) -> Result<(), String> {
	if visited.contains(card_id) {
		return Ok(());
	}
	if !visiting.insert(card_id.to_string()) {
		return Err(format!("cycle detected at {card_id}"));
	}

	if let Some(targets) = outgoing.get(card_id) {
		for target in targets {
			visit_for_cycles(target, outgoing, visiting, visited)?;
		}
	}

	visiting.remove(card_id);
	visited.insert(card_id.to_string());
	Ok(())
}

fn can_reach_root(start: &str, root_id: &str, outgoing: &HashMap<String, Vec<String>>) -> bool {
	let mut queue = VecDeque::from([start.to_string()]);
	let mut seen = HashSet::<String>::new();

	while let Some(card_id) = queue.pop_front() {
		if card_id == root_id {
			return true;
		}
		if !seen.insert(card_id.clone()) {
			continue;
		}
		if let Some(targets) = outgoing.get(&card_id) {
			for target in targets {
				queue.push_back(target.clone());
			}
		}
	}

	false
}

#[cfg(test)]
mod tests {
	use super::*;

	const ROOT_ID: &str = "driver:dumptruck-root";

	#[test]
	fn validate_model_accepts_simple_graph() {
		let mut cards = HashMap::new();
		cards.insert(
			ROOT_ID.to_string(),
			AuroraCard {
				id: ROOT_ID.to_string(),
				card_type: "driver".to_string(),
				name: "Root".to_string(),
			},
		);
		cards.insert(
			"requirement:one".to_string(),
			AuroraCard {
				id: "requirement:one".to_string(),
				card_type: "requirement".to_string(),
				name: "One".to_string(),
			},
		);

		let links = vec![AuroraLink {
			id: "link:one:root:test".to_string(),
			source: "requirement:one".to_string(),
			target: ROOT_ID.to_string(),
		}];

		validate_model(&cards, &links, ROOT_ID).expect("expected validation to succeed");
	}

	#[test]
	fn validate_model_rejects_unknown_card_reference() {
		let mut cards = HashMap::new();
		cards.insert(
			ROOT_ID.to_string(),
			AuroraCard {
				id: ROOT_ID.to_string(),
				card_type: "driver".to_string(),
				name: "Root".to_string(),
			},
		);
		cards.insert(
			"requirement:one".to_string(),
			AuroraCard {
				id: "requirement:one".to_string(),
				card_type: "requirement".to_string(),
				name: "One".to_string(),
			},
		);

		let links = vec![AuroraLink {
			id: "link:one:missing:test".to_string(),
			source: "requirement:one".to_string(),
			target: "requirement:missing".to_string(),
		}];

		let err = validate_model(&cards, &links, ROOT_ID).expect_err("expected failure");
		assert!(err.contains("missing target"));
	}

	#[test]
	fn validate_model_rejects_cycle() {
		let mut cards = HashMap::new();
		cards.insert(
			ROOT_ID.to_string(),
			AuroraCard {
				id: ROOT_ID.to_string(),
				card_type: "driver".to_string(),
				name: "Root".to_string(),
			},
		);
		for id in ["requirement:a", "requirement:b"] {
			cards.insert(
				id.to_string(),
				AuroraCard {
					id: id.to_string(),
					card_type: "requirement".to_string(),
					name: id.to_string(),
				},
			);
		}

		let links = vec![
			AuroraLink {
				id: "link:a:b:test".to_string(),
				source: "requirement:a".to_string(),
				target: "requirement:b".to_string(),
			},
			AuroraLink {
				id: "link:b:a:test".to_string(),
				source: "requirement:b".to_string(),
				target: "requirement:a".to_string(),
			},
		];

		let err = validate_model(&cards, &links, ROOT_ID).expect_err("expected failure");
		assert!(err.contains("cycle"));
	}

	#[test]
	fn validate_model_rejects_root_outgoing_link() {
		let mut cards = HashMap::new();
		cards.insert(
			ROOT_ID.to_string(),
			AuroraCard {
				id: ROOT_ID.to_string(),
				card_type: "driver".to_string(),
				name: "Root".to_string(),
			},
		);
		cards.insert(
			"requirement:one".to_string(),
			AuroraCard {
				id: "requirement:one".to_string(),
				card_type: "requirement".to_string(),
				name: "One".to_string(),
			},
		);

		let links = vec![AuroraLink {
			id: "link:root:one:test".to_string(),
			source: ROOT_ID.to_string(),
			target: "requirement:one".to_string(),
		}];

		let err = validate_model(&cards, &links, ROOT_ID).expect_err("expected failure");
		assert!(err.contains("root") && err.contains("outgoing"));
	}
}
