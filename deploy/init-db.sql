-- Create the pgvector extension if available
CREATE EXTENSION IF NOT EXISTS vector;

-- Create the Apache AGE extension (if installed)
CREATE EXTENSION IF NOT EXISTS age;

LOAD 'age';
SET search_path = ag_catalog, "$user", public;
SELECT create_graph('dumptruck');

-- Example schema/table for storing normalized rows and vector embeddings
-- Production-ready normalized rows schema with dedicated columns for
-- events, address/credential hashes, file provenance and indexing.
CREATE TABLE IF NOT EXISTS normalized_rows (
    id bigserial primary key,
    dataset text,
    event_type text,
    address_hash text,
    credential_hash text,
    row_hash text,
    file_id text,
    source_file text,
    fields jsonb,
    embedding vector(1536),
    created_at timestamptz default now()
);

-- Indexes to support fast lookups for address/credential detection.
CREATE INDEX IF NOT EXISTS idx_normalized_rows_address_hash ON normalized_rows(address_hash);
CREATE INDEX IF NOT EXISTS idx_normalized_rows_credential_hash ON normalized_rows(credential_hash);
CREATE INDEX IF NOT EXISTS idx_normalized_rows_file_id ON normalized_rows(file_id);
CREATE INDEX IF NOT EXISTS idx_normalized_rows_row_hash ON normalized_rows(row_hash);
CREATE INDEX IF NOT EXISTS idx_normalized_rows_source_file ON normalized_rows(source_file);

-- Canonical email addresses: SHA256 hash of normalized (NFKC + case-folded) form.
-- This is the primary key for deduplication and grouping.
CREATE TABLE IF NOT EXISTS canonical_addresses (
    canonical_hash text primary key,
    address_text text not null unique,
    -- Keep the original normalized form for reference/auditing
    normalized_form text,
    -- Vector embedding (Nomic via Ollama) for similarity-based deduplication
    -- Helps catch near-duplicates like typos, variations not caught by normalization
    embedding vector(768) default null,
    first_seen_at timestamptz default now(),
    updated_at timestamptz default now()
);

-- Unicode alternate representations of the same canonical address.
-- Maps variant Unicode forms (composed/decomposed, fullwidth, etc.)
-- to the canonical SHA256 hash.
CREATE TABLE IF NOT EXISTS address_alternates (
    id bigserial primary key,
    canonical_hash text not null references canonical_addresses(canonical_hash) on delete cascade,
    alternate_hash text not null,
    -- Store the alternate Unicode text for reference/debugging
    alternate_form text,
    first_seen_at timestamptz default now()
);

-- Create unique constraint to prevent duplicate alternate mappings
CREATE UNIQUE INDEX IF NOT EXISTS idx_address_alternates_canonical_alternate 
    ON address_alternates(canonical_hash, alternate_hash);

-- Vector similarity index (IVFFlat) for efficient nearest-neighbor search
-- Helps identify near-duplicate addresses (typos, variations) via Nomic embeddings
CREATE INDEX IF NOT EXISTS idx_canonical_addresses_embedding ON canonical_addresses 
    USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
-- Credentials associated with a canonical address.
-- Stores SHA256 hashes of (address, credential) pairs to track which
-- credentials have been seen with a given canonical address.
CREATE TABLE IF NOT EXISTS address_credentials (
    id bigserial primary key,
    canonical_hash text not null references canonical_addresses(canonical_hash) on delete cascade,
    credential_hash text not null,
    -- Count occurrences across all dumps
    occurrence_count integer default 1,
    first_seen_at timestamptz default now(),
    last_seen_at timestamptz default now()
);

-- Create unique constraint to prevent duplicate credential associations
CREATE UNIQUE INDEX IF NOT EXISTS idx_address_credentials_canonical_credential
    ON address_credentials(canonical_hash, credential_hash);

-- Index for fast lookup of credentials by address
CREATE INDEX IF NOT EXISTS idx_address_credentials_canonical_hash 
    ON address_credentials(canonical_hash);

-- Index for fast lookup of canonical addresses by alternate hash
CREATE INDEX IF NOT EXISTS idx_address_alternates_alternate_hash 
    ON address_alternates(alternate_hash);
-- Address co-occurrence: tracks how many times two canonical addresses
-- have been seen together in the same row (i.e., both present as credentials
-- or fields in a single dump record). This forms an undirected graph edge
-- with edge weights (occurrence counts), useful for reconstructing
-- "canonical dumps" and finding related identities.
-- To avoid storing both (A, B) and (B, A), we store them in canonical order:
-- addr_hash_1 < addr_hash_2 (lexicographically).
CREATE TABLE IF NOT EXISTS address_cooccurrence (
    id bigserial primary key,
    canonical_hash_1 text not null references canonical_addresses(canonical_hash) on delete cascade,
    canonical_hash_2 text not null references canonical_addresses(canonical_hash) on delete cascade,
    -- Count how many times these addresses appeared together
    cooccurrence_count integer default 1,
    -- Track when the pair was first and last observed
    first_seen_at timestamptz default now(),
    last_seen_at timestamptz default now(),
    -- Enforce addr_1 < addr_2 to avoid duplicate edges
    constraint addr_order check (canonical_hash_1 < canonical_hash_2)
);

-- Unique index to prevent duplicate pairs
CREATE UNIQUE INDEX IF NOT EXISTS idx_address_cooccurrence_pair
    ON address_cooccurrence(canonical_hash_1, canonical_hash_2);

-- Indexes for fast graph traversal (find neighbors of a node)
CREATE INDEX IF NOT EXISTS idx_address_cooccurrence_addr1
    ON address_cooccurrence(canonical_hash_1);
CREATE INDEX IF NOT EXISTS idx_address_cooccurrence_addr2
    ON address_cooccurrence(canonical_hash_2);

-- Have I Been Pwned (HIBP) breach data for canonical addresses.
-- Enriches canonical addresses with real-world breach intelligence.
CREATE TABLE IF NOT EXISTS address_breaches (
    id bigserial primary key,
    canonical_hash text not null references canonical_addresses(canonical_hash) on delete cascade,
    -- Name of the breach (e.g., "Adobe", "LinkedIn", "Facebook")
    breach_name text not null,
    -- Human-readable title of the breach
    breach_title text,
    -- Domain associated with the breach
    breach_domain text,
    -- Date the breach occurred (ISO 8601 format)
    breach_date date,
    -- When HIBP added knowledge of this breach
    added_date timestamptz,
    -- When the breach record was last modified
    modified_date timestamptz,
    -- How many users were affected in this breach
    pwn_count integer,
    -- Description of the breach (may be lengthy)
    description text,
    -- Whether the breach is verified by the organization
    is_verified boolean,
    -- Whether the breach appears to be fabricated
    is_fabricated boolean,
    -- Whether the breach is considered sensitive
    is_sensitive boolean,
    -- Whether the breach has been retired
    is_retired boolean,
    -- URL to the breach logo/icon
    logo_path text,
    -- When this address was last checked against HIBP
    checked_at timestamptz default now(),
    -- When this record was created
    first_seen_at timestamptz default now()
);

-- Unique index to prevent duplicate breach records for same canonical hash
CREATE UNIQUE INDEX IF NOT EXISTS idx_address_breaches_unique
    ON address_breaches(canonical_hash, breach_name);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_address_breaches_canonical_hash
    ON address_breaches(canonical_hash);
CREATE INDEX IF NOT EXISTS idx_address_breaches_breach_name
    ON address_breaches(breach_name);
CREATE INDEX IF NOT EXISTS idx_address_breaches_checked_at
    ON address_breaches(checked_at);