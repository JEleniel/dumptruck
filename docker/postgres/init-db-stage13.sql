-- Stage 13: Storage Enhancement Schema Extensions
--
-- These tables support the advanced pipeline stages:
-- - Stage 1: Evidence Preservation (file_metadata)
-- - Stage 4: Chain of Custody (chain_of_custody_records)
-- - Stage 8: Alias Resolution (alias_relationships)
-- - Stage 10: Anomaly Detection (anomaly_scores)

-- ==============================================================================
-- File Metadata Table (Stage 1: Evidence Preservation)
-- ==============================================================================
-- Tracks cryptographic evidence for all ingested files.
-- Supports:
-- - SHA-256 hash for integrity verification
-- - BLAKE3 hash for faster/parallel hashing (future optimization)
-- - Alternate filenames tracking
-- - Complete chain of custody trail
--
CREATE TABLE IF NOT EXISTS file_metadata (
	id bigserial primary key,
	-- Unique file identifier (UUID v4 + Unix timestamp)
	file_id text not null unique,
	-- Original filename
	original_filename text not null,
	-- SHA-256 hash of file contents (hex-encoded)
	sha256_hash text not null,
	-- BLAKE3 hash of file contents (hex-encoded, for dual-hash verification)
	blake3_hash text,
	-- File size in bytes
	file_size bigint not null,
	-- Alternate names the file has been known by (JSONB array)
	alternate_names jsonb default '[]'::jsonb,
	-- Timestamp when file was first seen
	created_at timestamptz default now(),
	-- Timestamp when file was ingested
	ingested_at timestamptz,
	-- Processing status: pending, processing, completed, error
	processing_status text default 'pending'
);

-- Indexes for fast file lookups
CREATE INDEX IF NOT EXISTS idx_file_metadata_file_id ON file_metadata(file_id);
CREATE INDEX IF NOT EXISTS idx_file_metadata_sha256_hash ON file_metadata(sha256_hash);
CREATE INDEX IF NOT EXISTS idx_file_metadata_blake3_hash ON file_metadata(blake3_hash);
CREATE INDEX IF NOT EXISTS idx_file_metadata_original_filename ON file_metadata(original_filename);
CREATE INDEX IF NOT EXISTS idx_file_metadata_created_at ON file_metadata(created_at);

-- ==============================================================================
-- Chain of Custody Records Table (Stage 4: Chain of Custody)
-- ==============================================================================
-- Cryptographically signed audit trail for forensic compliance.
-- Each record is signed with ED25519 to create an immutable chain.
-- Enables verification that data processing was authorized and unmodified.
--
CREATE TABLE IF NOT EXISTS chain_of_custody_records (
	id bigserial primary key,
	-- Reference to file being processed (foreign key)
	file_id text not null references file_metadata(file_id) on delete cascade,
	-- Unique record identifier
	record_id text not null unique,
	-- Custody action: FileIngested, FileValidated, DuplicationCheck, etc.
	custody_action text not null,
	-- Operator/user who performed the action
	operator text not null,
	-- SHA-256 hash of the file at time of this action
	file_hash text not null,
	-- ED25519 signature of (file_id || custody_action || operator || file_hash || timestamp)
	signature bytea not null,
	-- Public key used to verify signature
	public_key bytea not null,
	-- Number of records/rows processed in this action
	record_count integer default 0,
	-- Optional notes about the action
	notes text,
	-- Timestamp when action occurred
	action_timestamp timestamptz default now(),
	-- When this record was created in the database
	created_at timestamptz default now()
);

-- Indexes for audit trail queries
CREATE INDEX IF NOT EXISTS idx_chain_of_custody_file_id ON chain_of_custody_records(file_id);
CREATE INDEX IF NOT EXISTS idx_chain_of_custody_record_id ON chain_of_custody_records(record_id);
CREATE INDEX IF NOT EXISTS idx_chain_of_custody_operator ON chain_of_custody_records(operator);
CREATE INDEX IF NOT EXISTS idx_chain_of_custody_action ON chain_of_custody_records(custody_action);
CREATE INDEX IF NOT EXISTS idx_chain_of_custody_timestamp ON chain_of_custody_records(action_timestamp);

-- ==============================================================================
-- Alias Relationships Table (Stage 8: Alias Resolution)
-- ==============================================================================
-- Tracks related entries across multiple identity formats.
-- Links:
-- - Email variants (plus addressing, dots)
-- - User ID variants
-- - Phone number format variations
-- - National ID variations
-- Each link includes confidence score (0-100) indicating relationship strength.
--
CREATE TABLE IF NOT EXISTS alias_relationships (
	id bigserial primary key,
	-- The "canonical" form (preferred representation)
	canonical_value text not null,
	-- Hash of canonical form for faster lookups
	canonical_hash text not null,
	-- The variant/alternate form
	variant_value text not null,
	-- Hash of variant form
	variant_hash text not null,
	-- Type of alias: EmailPlus, EmailDot, EmailDomain, PhoneNormalization, etc.
	alias_type text not null,
	-- Confidence score (0-100): how certain we are this is a true alias
	confidence integer not null check (confidence >= 0 and confidence <= 100),
	-- Optional metadata about the alias detection (JSON)
	metadata jsonb default '{}'::jsonb,
	-- When this relationship was discovered
	discovered_at timestamptz default now(),
	-- When this relationship was last verified/updated
	verified_at timestamptz default now()
);

-- Indexes for alias lookups
CREATE INDEX IF NOT EXISTS idx_alias_relationships_canonical_hash ON alias_relationships(canonical_hash);
CREATE INDEX IF NOT EXISTS idx_alias_relationships_variant_hash ON alias_relationships(variant_hash);
CREATE INDEX IF NOT EXISTS idx_alias_relationships_alias_type ON alias_relationships(alias_type);
CREATE INDEX IF NOT EXISTS idx_alias_relationships_confidence ON alias_relationships(confidence);
-- Unique constraint prevents duplicate alias relationships
CREATE UNIQUE INDEX IF NOT EXISTS idx_alias_relationships_unique
	ON alias_relationships(canonical_hash, variant_hash, alias_type);

-- ==============================================================================
-- Anomaly Scores Table (Stage 10: Anomaly & Novelty Detection)
-- ==============================================================================
-- Records anomalies and outliers detected in breach data.
-- Supports:
-- - Entropy outliers (>3 standard deviations)
-- - Unseen field combinations
-- - Rare domain/user detection
-- - Unusual password format detection
-- - Baseline deviation detection
-- Each anomaly includes risk score (0-100) for severity assessment.
--
CREATE TABLE IF NOT EXISTS anomaly_scores (
	id bigserial primary key,
	-- Reference to file where anomaly was detected
	file_id text not null references file_metadata(file_id) on delete cascade,
	-- The subject being analyzed (address, credential, domain, etc.)
	subject_hash text not null,
	-- Type of anomaly: EntropyOutlier, UnseenCombination, RareDomain, etc.
	anomaly_type text not null,
	-- Risk score (0-100): severity of the anomaly
	risk_score integer not null check (risk_score >= 0 and risk_score <= 100),
	-- The specific metric that triggered the anomaly (JSON)
	-- e.g., {"entropy": 4.2, "threshold": 3.5} for entropy outlier
	metric jsonb default '{}'::jsonb,
	-- The computed threshold that was exceeded
	threshold_value real,
	-- The actual computed value that exceeded the threshold
	actual_value real,
	-- Whether this anomaly has been investigated/resolved
	is_resolved boolean default false,
	-- Optional notes about the anomaly
	notes text,
	-- When the anomaly was first detected
	detected_at timestamptz default now(),
	-- When it was investigated/resolved
	resolved_at timestamptz
);

-- Indexes for anomaly queries
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_file_id ON anomaly_scores(file_id);
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_subject_hash ON anomaly_scores(subject_hash);
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_anomaly_type ON anomaly_scores(anomaly_type);
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_risk_score ON anomaly_scores(risk_score);
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_detected_at ON anomaly_scores(detected_at);
CREATE INDEX IF NOT EXISTS idx_anomaly_scores_is_resolved ON anomaly_scores(is_resolved);
