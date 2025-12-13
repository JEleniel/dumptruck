# Dumptruck Complete Pipeline Diagram

## Overall Pipeline Architecture

```mermaid
graph TD
    A["📥 Input Data<br/>CSV/JSON/YAML"] -->|Parse & Extract| B["Records<br/>Multiple Fields"]
    B -->|Each Address Field| C["🔤 Normalization<br/>NFKC + ICU4X Case-Fold<br/>+ Punctuation Map"]
    C -->|Normalized Form| D["#️⃣ Compute Hash<br/>SHA256<br/>canonical_hash"]
    
    D -->|Hash + Text| E["🔍 Check Duplicate"]
    E -->|Exact Hash Match?| E1["✅ Found<br/>Link to Canonical"]
    E -->|No Match| E2["➡️ Continue"]
    
    E2 -->|Address Text| F["🧠 Generate Embedding<br/>Ollama/Nomic API<br/>nomic-embed-text:latest<br/>768-dim vector"]
    F -->|Embedding Vector| G["🔎 Vector Similarity<br/>pgvector Cosine<br/>threshold: 0.85"]
    G -->|Similar Match?| G1["✅ Found<br/>Link to Canonical"]
    G -->|No Match| G2["➡️ New Address"]
    
    G2 -->|Query HIBP| H["🚨 HIBP API v3<br/>GET /breachedaccount<br/>API Key:<br/>00000000000000000000000000000000"]
    H -->|Breach Data| I["💾 Store Breaches<br/>address_breaches Table<br/>breach_name, pwn_count,<br/>is_verified, etc."]
    
    I -->|Store| J["🗄️ PostgreSQL"]
    J -->|canonical_addresses| J1["canonical_hash PK<br/>address_text<br/>embedding vector<br/>normalized_form"]
    J -->|address_breaches| J2["canonical_hash FK<br/>breach_name<br/>pwn_count<br/>breach_date"]
    J -->|address_alternates| J3["canonical_hash FK<br/>alternate_hash<br/>Unicode variants"]
    J -->|address_credentials| J4["canonical_hash FK<br/>credential_hash<br/>occurrence_count"]
    J -->|address_cooccurrence| J5["hash_1, hash_2<br/>cooccurrence_count<br/>Graph edges"]
    
    E1 -->|Track Alternate| K["Track Unicode<br/>Variants<br/>Composed/Decomposed<br/>Fullwidth/ASCII"]
    K -->|Store| J3
    
    G1 -->|Track Alternate| K
    
    L["Credential Hash<br/>MD5/SHA256"] -->|Link| M["address_credentials<br/>occurrence_count++"]
    M -->|Store| J4
    
    N["Multiple Addresses<br/>in Record"] -->|Pair Detection| O["address_cooccurrence<br/>Undirected Edges<br/>hash_1 < hash_2"]
    O -->|Store| J5
```

## Detailed Component Flow

### 1. Input & Parsing

```mermaid
graph LR
    A["Raw Data<br/>CSV/JSON"] --> B["Parser<br/>CSV/JSON Adapter"]
    B --> C["Structured Records<br/>Field Extraction"]
    C --> D["Address Fields<br/>Credential Fields<br/>Other Context"]
```

### 2. Normalization Pipeline

```mermaid
graph TD
    A["Raw Address<br/>john.doe@EXAMPLE.COM<br/>José@example.com"] -->|Trim| B["Trim<br/>john.doe@EXAMPLE.COM"]
    B -->|NFKC Decompose| C["Decompose<br/>jose@EXAMPLE.COM"]
    C -->|ICU4X Case-Fold| D["Case-Fold<br/>jose@example.com"]
    D -->|Map Punctuation| E["Map Punct<br/>O'Connor → o'connor"]
    E -->|Collapse Whitespace| F["Collapsed<br/>jose@example.com"]
    F -->|Final Trim| G["✅ Canonical Form<br/>jose@example.com"]
```

### 3. Deduplication Check

```mermaid
graph TD
    A["Canonical Hash<br/>SHA256(normalized)"] --> B{"Hash<br/>Exists?"}
    B -->|Yes| C["✅ Duplicate Found<br/>Link to Canonical"]
    B -->|No| D["Generate Embedding<br/>Ollama/Nomic"]
    D --> E["Vector Similarity<br/>pgvector cosine"]
    E --> F{"Similarity<br/>≥ 0.85?"}
    F -->|Yes| G["✅ Duplicate Found<br/>Vector Match"]
    F -->|No| H["➡️ New Canonical<br/>Address"]
```

### 4. Enrichment: Vector Embedding

```mermaid
graph LR
    A["Address Text<br/>john.doe@example.com"] -->|HTTP POST| B["Ollama API<br/>nomic-embed-text:latest"]
    B -->|768-dim Vector| C["Embedding<br/>[0.234, -0.456, ...]"]
    C -->|pgvector<br/>IVFFlat Index| D["Canonical Addresses<br/>embedding column"]
    D -->|Cosine Distance| E["Find Similar<br/>threshold: 0.85"]
```

### 5. Enrichment: HIBP Breach Data

```mermaid
graph LR
    A["Canonical Address<br/>john.doe@example.com"] -->|Query| B["HIBP API v3<br/>/breachedaccount/{email}"]
    B -->|Headers:<br/>User-Agent + API Key| C["API Key:<br/>000000...0000<br/>Test Key"]
    C -->|HTTP Response| D["Breach Array<br/>name, title, domain,<br/>breach_date, pwn_count"]
    D -->|Store| E["address_breaches<br/>Table"]
    E -->|Indexes| F["Fast Lookup:<br/>By canonical_hash<br/>By breach_name<br/>By checked_at"]
```

### 6. Co-occurrence Tracking

```mermaid
graph TD
    A["Record: john.doe@example.com, password123,<br/>jane.smith@example.com, secret456"] -->|Parse| B["Address Pair<br/>john.doe<br/>jane.smith"]
    B -->|Canonicalize| C["Canonical Hashes<br/>hash_john, hash_jane"]
    C -->|Order Pair<br/>hash_1 < hash_2| D["Undirected Edge<br/>hash_john, hash_jane"]
    D -->|Insert/Increment| E["address_cooccurrence<br/>cooccurrence_count++"]
    E -->|Graph Query| F["Get Neighbors<br/>Find all addresses<br/>seen with address X"]
```

## Complete Database Schema

```mermaid
erDiagram
    canonical_addresses ||--o{ address_alternates : has
    canonical_addresses ||--o{ address_credentials : contains
    canonical_addresses ||--o{ address_breaches : found_in
    canonical_addresses ||--o{ address_cooccurrence : participates_in
    normalized_rows ||--o{ canonical_addresses : references
    
    canonical_addresses {
        string canonical_hash PK
        string address_text UK
        string normalized_form
        vector embedding
        timestamp first_seen_at
        timestamp updated_at
    }
    
    address_alternates {
        bigint id PK
        string canonical_hash FK
        string alternate_hash
        string alternate_form
        timestamp first_seen_at
    }
    
    address_credentials {
        bigint id PK
        string canonical_hash FK
        string credential_hash
        int occurrence_count
        timestamp first_seen_at
        timestamp last_seen_at
    }
    
    address_breaches {
        bigint id PK
        string canonical_hash FK
        string breach_name
        string breach_title
        string breach_domain
        date breach_date
        int pwn_count
        boolean is_verified
        boolean is_fabricated
        timestamp checked_at
    }
    
    address_cooccurrence {
        bigint id PK
        string canonical_hash_1 FK
        string canonical_hash_2 FK
        int cooccurrence_count
        timestamp first_seen_at
        timestamp last_seen_at
    }
    
    normalized_rows {
        bigint id PK
        string dataset
        string event_type
        string address_hash
        string credential_hash
        string row_hash
        string file_id
        string source_file
        jsonb fields
    }
```

## Processing Statistics

```mermaid
graph LR
    A["1,000 Records<br/>Input"] --> B["~5,000 Addresses<br/>Extract"]
    B --> C["~4,000 Unique<br/>Normalize"]
    C --> D["~3,500 New<br/>Generate Embeddings"]
    D --> E["~100 Duplicates<br/>Vector Match"]
    E --> F["~3,400 Canonical<br/>Addresses"]
    F --> G["~500 Breach<br/>Matches"]
```

## Performance Characteristics

```mermaid
graph TD
    A["Operation"] --> B["Latency"]
    A --> C["Throughput"]
    
    B --> B1["Normalization: < 1ms"]
    B --> B2["Hash Compute: < 1ms"]
    B --> B3["Embedding Gen: 100-200ms"]
    B --> B4["Vector Search: 10-50ms"]
    B --> B5["HIBP Query: 200-500ms"]
    
    C --> C1["Single-threaded: 3-5 addr/sec"]
    C --> C2["10x concurrent: 50-100 addr/sec"]
    C --> C3["With HIBP API key: 10x faster"]
```

## Data Flow Summary

```mermaid
graph LR
    A["🗂️ Input"] --> B["📋 Parse"]
    B --> C["🔤 Normalize"]
    C --> D["#️⃣ Hash"]
    D --> E["🔍 Dedup"]
    E --> F["🧠 Embed"]
    F --> G["🚨 HIBP"]
    G --> H["💾 Store"]
    H --> I["📊 Output"]
```
