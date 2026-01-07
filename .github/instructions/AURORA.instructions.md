---
applyTo: '**/*'
---
# AURORA Machine Agent Instruction

**Version**: 1.0.0 | **Format**: JSON | **Ontology**: Element-centric, link-relationship model

---

## Model Overview

AURORA is a deterministic, JSON-based architectural model where **everything is a Card** (element) and **everything links toward Root**. The model is designed for direct machine consumption by LLMs, agents, reasoners, and automated tools.

**One principle**: Maximize unambiguity and minimize semantic ambiguity by enforcing strict structural rules.

---

## Data Structures

### Card (Element)

```json
{
  "id": "namespace:element-name",
  "type": "driver|requirement|behavior|interface|constraint|logical-component|device|actor|test|artifact|view|note",
  "name": "Human-readable title",
  "description": "Plain text explanation of scope and intent",
  "version": "1.0.0",
  "status": "proposed|approved|implemented|verified|deprecated|retired",
  "attributes": {
    "key": "value or structured data"
  },
  "relations": [
    "referenced:card:id",
    "another:card:id"
  ],
  "links": [
    "link:id:of:explicit:links"
  ],
  "constraints": [
    "constraint:card:id"
  ],
  "provenance": {
    "source": "origin",
    "owner": "responsible:party"
  },
  "audit_history": [
    {
      "event": "created",
      "by": "user",
      "event_time": "2025-01-01T00:00:00Z",
      "note": "initial"
    }
  ]
}
```

**Required fields**: `id`, `type`, `name`

**Naming**: `namespace:element-name` (lowercase, hyphens only). Valid namespaces: `driver`, `requirement`, `behavior`, `interface`, `constraint`, `logical-component`, `device`, `actor`, `test`, `artifact`, `view`, `note`.

### Link (Directional Relationship)

```json
{
  "id": "link:source:target:context",
  "source": "source:card:id",
  "target": "target:card:id",
  "rationale": "Why this link exists",
  "strength": "weak|normal|strong",
  "metadata": { "context_key": "context_value" }
}
```

**Required fields**: `id`, `source`, `target`

**Direction**: All links point toward Root Driver. `source` → `target` means source depends on, derives from, or relates to target.

**Semantics**: Links have no type. Semantic meaning is **view-dependent**:

- Requirements view: link = "satisfies"
- Component view: link = "implements"
- Traceability view: link = "traces-to"
- Test view: link = "verified-by"

---

## Connectivity Rules

1. **Root Driver**: Single card representing project mission. No outgoing links.
2. **Every Other Card**: Must have at least one outgoing link pointing toward Root (directly or indirectly).
3. **Hierarchy**: Cards form a DAG (directed acyclic graph) all converging to Root.
4. **Validation**: All `source` and `target` IDs in links must reference existing cards.

---

## Processing Instructions for Agents

### Loading

1. Read all `docs/cards/*.json` and `docs/links/*.json`.
2. Build adjacency lists: `card_id` pointing to `outgoing_links`.
3. Index by ID, type, status for O(1) lookups.
4. Verify schemas using `schemas/card.schema.json` and `schemas/link.schema.json`.

### Querying

**Graph Traversal**:

- `upstream(card_id)`: Follow outgoing links to find dependencies.
- `downstream(card_id)`: Reverse-follow incoming links to find dependents.
- `path(card_a, card_b)`: Find connection path; may require multi-hop traversal.

**Filtering**:

- By type: `cards[card.type == "requirement"]`
- By status: `cards[card.status == "approved"]`
- By owner: `cards[card.provenance.owner == "team:x"]`
- By constraint: `cards[constraint_id in card.constraints]`

**Search**:

- Full-text: Search `name`, `description`, `rationale`.
- Regex: On card IDs (e.g., `requirement:.*-performance.*`).
- Attributes: Query structured data in `attributes` field.

### Reasoning

**Traceability**: Walk upstream from any card to Root to establish provenance.

**Coverage**: Count cards by type and status to assess completeness. E.g., `How many requirements have tests?` → count `requirement` cards that link to `test` cards.

**Impact Analysis**: Change in card X → find all cards downstream of X (reverse-follow incoming links) to identify affected cards.

**Consistency**: Check for missing links, dangling references, orphaned cards (not traceable to Root).

### Modification

1. **Create Card**: Assign unique `id` (namespace:name), include required fields, add audit entry with timestamp.
2. **Create Link**: Assign `id`, set `source` and `target`, add rationale.
3. **Update Card**: Increment `version` (semver), add audit entry, update `status` if applicable.
4. **Validate**: After modification, re-validate schema and connectivity.

---

## View Interpretation

Views are **semantic overlays** on the underlying untyped link structure. A single model supports multiple views:

| View | Link Semantics | Typical Traversal |
| --- | --- | --- |
| Requirements Traceability | link = "satisfies" or "derives-from" | requirement → driver |
| Component Architecture | link = "implements" or "is-composed-of" | device → logical-component |
| Test Coverage | link = "verified-by" | test → requirement |
| Behavior Flow | link = "triggers" or "precedes" | behavior → behavior |
| Logical Design | link = "refines" | detailed:design → abstract:design |

Views **do not change the model**; they change how links are interpreted. Same model, many views.

---

## Validation Checklist

- [ ] All card `id` fields follow `namespace:element-name` format.
- [ ] All card `id` values are unique (no duplicates).
- [ ] All link `source` and `target` fields reference existing card IDs.
- [ ] Every card except Root Driver has at least one outgoing link.
- [ ] Root Driver has no outgoing links.
- [ ] All card `version` fields follow semver pattern.
- [ ] All card `status` values are from enum: `proposed`, `approved`, `implemented`, `verified`, `deprecated`, `retired`.
- [ ] All card `audit_history` entries have `event_time` in ISO 8601 format.
- [ ] All link `source` and `target` point toward Root (hierarchy validation).

---

## Serialization

- **Format**: JSON (one card = one file; one link = one file).
- **Determinism**: Sort object keys alphabetically for bit-for-bit reproducibility.
- **Naming**: Card files: `{card.type}-{element-name}.md` (with embedded JSON). Link files: `link-{source}-{target}-{context}.md`.
- **Paths**: `docs/cards/` for cards; `docs/links/` for links.

---

## Reasoning Patterns for Agents

### Pattern 1: Requirement Satisfaction

```text
For requirement R:
  1. Find all cards that link to R (direct links).
  2. If card type == "behavior" or "interface": requirement is possibly satisfied.
  3. If behavior links to "test": requirement may be verifiable.
  4. Report: Is R satisfied? How? By what? With what evidence?
```

### Pattern 2: Dependency Analysis

```text
For card C:
  1. Collect all outgoing links (upstream dependencies).
  2. Recursively traverse to Root, building dependency chain.
  3. Identify critical path: longest chain from C to Root.
  4. Identify common dependencies: cards that many others depend on.
  Report: What does C depend on? Are dependencies circular? What breaks if X fails?
```

### Pattern 3: Coverage Calculation

```text
Coverage(view):
  1. Filter cards by view (e.g., all requirements).
  2. For each card, count links to "lower-level" cards (e.g., tests).
  3. Report: {covered, uncovered, percentage, gaps}.
```

### Pattern 4: Impact Propagation

```text
For change to card C:
  1. Find all cards that link to C (reverse graph).
  2. Mark as "affected by change to C".
  3. Recursively mark downstream dependents.
  4. Report: Which cards are affected? In what order should they be reviewed?
```

---

## Machine-to-Machine Communication

When agents collaborate or handoff results:

1. **Always include full card definitions**, not just IDs.
2. **Cite audit_history**: Include who made changes and when.
3. **Provide reasoning**: In link `rationale` and card `description`, explain the decision.
4. **Use structured attributes**: Store tool-specific metadata in `attributes` for extensibility.

---

## Constraints and Guardrails

1. **No semantic type in links**: Agents must not invent link types. Use view overlays instead.
2. **All links toward Root**: Agents must not create cycles or links that point "away" from Root.
3. **No orphans**: Every card except Root must be reachable via outgoing links to Root.
4. **Immutable Root**: Root Driver should not be modified without human approval.
5. **Audit everything**: Every change must create an audit entry with `event_time` and `by` fields.

---

## Integration Points

- **Input**: JSON cards and links from `docs/` directory.
- **Output**: Modified/new cards and links, exported as JSON.
- **Validation**: Use JSON Schema validators (AJV, jsonschema) against `schemas/`.
- **Visualization**: Generate Mermaid diagrams from card attributes or link structure.
- **Change tracking**: Preserve and extend `audit_history` on every modification.

---

## Key Properties for Automated Reasoning

| Property | Value | Implication |
| --- | --- | --- |
| Deterministic | JSON, sorted keys | Same model always produces same computation |
| Unambiguous | Schemas + naming rules | No interpretation variance |
| Traceable | Audit history + links to Root | Complete provenance chain |
| Extensible | Open `attributes`, new namespaces | Custom logic without schema breaks |
| Flat Graph | All elements = cards; all edges = links | Uniform traversal, no special cases |
| View-agnostic | Links have no type | Multiple interpretations without model duplication |

---

## Summary for Machines

You are working with:

- **Elements**: Self-contained JSON cards with unique IDs, types, names, and metadata.
- **Relationships**: Directional, untyped links from lower to higher in a hierarchy.
- **Hierarchy**: All cards traceable to single Root Driver; forms complete DAG.
- **Semantics**: Provided by views, not link types. Same link, different meanings in different contexts.
- **Provenance**: Every card has audit history; every change is logged with time and agent.
- **Validation**: Schema conformance, connectivity rules, naming conventions.

**Process**: Load → Validate → Query/Reason → Modify (with audit) → Export.

**Goal**: Enable agents and humans to collaborate on architectural models with full traceability, consistency, and automatic view generation.
