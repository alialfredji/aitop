# Data Layer Evolution Plan

## Current Architecture (v0.2)

```
~/.claude/projects/**/*.jsonl
        │
        ▼
   ┌─────────┐     ┌──────────┐     ┌────────────┐
   │ Scanner  │────▶│  Parser  │────▶│  Database   │
   │ (walks)  │     │ (JSONL)  │     │  (SQLite)   │
   └─────────┘     └──────────┘     └────────────┘
                                          │
   ┌─────────┐                           │
   │ Watcher  │──── triggers ────────────┘
   │ (notify) │                           │
   └─────────┘                           ▼
                                    ┌────────────┐     ┌─────────┐
                                    │ Aggregator │────▶│   UI    │
                                    │ (SQL read) │     │ (views) │
                                    └────────────┘     └─────────┘
```

**Strengths:** Incremental indexing, WAL mode for concurrent access, instant subsequent
startups, bounded memory usage.

**Limitations:** Single data source (Claude Code JSONL), hardcoded pricing, no schema
migrations, single-threaded parsing, no data export.

---

## Phase 1: Provider Abstraction (v0.3)

### Goal: Decouple data ingestion from Claude Code specifics

#### 1.1 Provider Trait

```rust
/// A data source that can discover and parse AI usage data.
pub trait Provider: Send + Sync {
    /// Human-readable provider name (e.g., "Claude Code", "Admin API")
    fn name(&self) -> &str;

    /// Unique provider identifier for DB tagging
    fn id(&self) -> &str;

    /// Discover available data files/endpoints
    fn scan(&self) -> Result<Vec<DataSource>>;

    /// Parse new data from a source, starting at the given byte offset.
    /// Returns parsed messages and the new offset.
    fn parse(&self, source: &DataSource, offset: u64) -> Result<(Vec<ParsedMessage>, u64)>;

    /// Pricing table for cost calculation
    fn pricing(&self) -> &PricingTable;

    /// Optional: set up file/endpoint watching
    fn watch(&self) -> Option<Box<dyn Watcher>>;
}

pub struct DataSource {
    pub id: String,           // Unique identifier (file path or API endpoint)
    pub provider: String,     // Provider ID
    pub metadata: HashMap<String, String>,
}

pub struct PricingTable {
    pub models: HashMap<String, ModelPricing>,
}

pub struct ModelPricing {
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub cache_read_per_mtok: f64,
    pub cache_creation_per_mtok: f64,
}
```

#### 1.2 Refactor Current Code

```
src/data/
├── mod.rs
├── provider.rs          # Provider trait + PricingTable
├── providers/
│   ├── mod.rs
│   ├── claude_code.rs   # Current scanner.rs + parser.rs logic
│   ├── admin_api.rs     # Anthropic Admin API (v0.3)
│   └── otel.rs          # OpenTelemetry export (v0.3+)
├── db.rs                # Unchanged (provider-agnostic)
├── aggregator.rs        # Add provider filter param
└── watcher.rs           # Generalized watcher trait
```

#### 1.3 Database Schema Changes

```sql
-- Add provider column to sessions and messages
ALTER TABLE sessions ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude_code';
ALTER TABLE messages ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude_code';

-- Provider-aware file index
ALTER TABLE file_index ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude_code';

-- Index for provider filtering
CREATE INDEX idx_sessions_provider ON sessions(provider);
CREATE INDEX idx_messages_provider ON messages(provider);

-- Schema version tracking
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

---

## Phase 2: Dynamic Pricing (v0.3)

### Goal: Remove hardcoded pricing, support new models automatically

#### 2.1 Pricing Configuration

```toml
# ~/.config/aitop/pricing.toml (auto-generated, user-overridable)

[models.claude-opus-4-6]
input_per_mtok = 15.0
output_per_mtok = 75.0
cache_read_per_mtok = 1.50
cache_creation_per_mtok = 18.75

[models.claude-sonnet-4-6]
input_per_mtok = 3.0
output_per_mtok = 15.0
cache_read_per_mtok = 0.30
cache_creation_per_mtok = 3.75

[models.claude-haiku-4-5]
input_per_mtok = 0.80
output_per_mtok = 4.0
cache_read_per_mtok = 0.08
cache_creation_per_mtok = 1.0
```

#### 2.2 Pricing Lookup Chain

1. User override in `pricing.toml` (highest priority)
2. Built-in defaults (shipped with binary)
3. Fuzzy model name matching: `opus-4-6` matches `claude-opus-4-6-20260301`

#### 2.3 Pricing Update Command

```bash
aitop --update-pricing
# Fetches latest from a bundled JSON or user-specified URL
# Writes to ~/.config/aitop/pricing.toml
```

---

## Phase 3: Schema Migrations (v0.3)

### Goal: Safely evolve DB schema across versions

#### 3.1 Migration System

```rust
pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub up: &'static str,    // SQL to apply
    pub down: &'static str,  // SQL to rollback (best-effort)
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema",
        up: "CREATE TABLE sessions (...); CREATE TABLE messages (...); ...",
        down: "DROP TABLE sessions; DROP TABLE messages; ...",
    },
    Migration {
        version: 2,
        description: "Add provider columns",
        up: "ALTER TABLE sessions ADD COLUMN provider TEXT NOT NULL DEFAULT 'claude_code'; ...",
        down: "-- SQLite doesn't support DROP COLUMN, rebuild required",
    },
    Migration {
        version: 3,
        description: "Add anomaly flags",
        up: "ALTER TABLE messages ADD COLUMN anomaly INTEGER DEFAULT 0; ...",
        down: "",
    },
];
```

#### 3.2 Migration Runner

```rust
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current = get_schema_version(conn)?;
    for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
        conn.execute_batch(migration.up)?;
        set_schema_version(conn, migration.version)?;
        info!("Applied migration {}: {}", migration.version, migration.description);
    }
    Ok(())
}
```

- Run on every startup, idempotent
- Wrap each migration in a transaction
- Log applied migrations

---

## Phase 4: Performance (v0.4)

### 4.1 Parallel Parsing

```rust
use rayon::prelude::*;

pub fn initial_index(files: Vec<SessionFile>, db: &Database) -> Result<()> {
    let parsed: Vec<_> = files
        .par_iter()
        .map(|f| parse_file(f))
        .collect::<Result<Vec<_>>>()?;

    // Single-threaded DB writes (SQLite limitation)
    for batch in parsed {
        db.upsert_batch(&batch)?;
    }
    Ok(())
}
```

- Parse JSONL files in parallel across CPU cores
- Collect results, then write to DB sequentially (SQLite single-writer)
- Target: 4x speedup on initial index

### 4.2 Prepared Statement Cache

```rust
pub struct Aggregator {
    conn: Connection,
    stmts: HashMap<&'static str, Statement<'_>>,
}
```

- Pre-compile frequently-used SQL statements
- Avoid re-parsing SQL on every refresh cycle
- Measure: profile query time before/after

### 4.3 Query Result Caching

```rust
pub struct CachedAggregator {
    inner: Aggregator,
    cache: HashMap<String, (Instant, Box<dyn Any>)>,
    ttl: Duration,
}
```

- Cache query results for configurable TTL (default: refresh interval)
- Invalidate on file watcher events
- Reduces DB reads during rapid refresh cycles

### 4.4 Memory-Mapped Parsing

For very large JSONL files (>100MB):

```rust
use memmap2::MmapOptions;

let file = File::open(path)?;
let mmap = unsafe { MmapOptions::new().map(&file)? };
let data = &mmap[offset..];
// Parse directly from memory-mapped region
```

- Avoid reading entire files into heap memory
- OS manages page cache efficiently
- Only activate for files > threshold (e.g., 10MB)

---

## Phase 5: Data Export Pipeline (v0.3-v0.4)

### 5.1 Export Trait

```rust
pub trait Exporter {
    fn format(&self) -> &str;
    fn export(&self, data: &ExportData, writer: &mut dyn Write) -> Result<()>;
}

pub struct ExportData {
    pub sessions: Vec<SessionSummary>,
    pub models: Vec<ModelStats>,
    pub daily_spend: Vec<DailySpend>,
    pub dashboard: DashboardStats,
    pub anomalies: Vec<Anomaly>,
    pub generated_at: DateTime<Utc>,
    pub period: DateRange,
}
```

### 5.2 Export Formats

| Format | CLI Flag | Use Case |
|--------|----------|----------|
| JSON | `--export json` | Programmatic consumption, piping to jq |
| CSV | `--export csv` | Spreadsheet import, data analysis |
| Markdown | `--export markdown` | Reports, Slack/email, documentation |
| TSV | `--export tsv` | Unix pipeline integration |

### 5.3 Export Filters

```bash
aitop --export json --project echopad --since 2026-03-01 --until 2026-03-19
aitop --export markdown --this-week
aitop --export csv --model opus
```

---

## Phase 6: Data Integrity (v0.4+)

### 6.1 Deduplication

Problem: Admin API data may overlap with local JSONL data for the same sessions.

Strategy:
- Primary key: `(session_id, message_uuid)`
- If both providers report the same message, prefer local JSONL (higher fidelity)
- Track `source_priority` per provider

### 6.2 Data Validation

```rust
pub fn validate_message(msg: &ParsedMessage) -> ValidationResult {
    let mut warnings = vec![];

    if msg.cost_usd < 0.0 {
        warnings.push("Negative cost");
    }
    if msg.input_tokens == 0 && msg.output_tokens == 0 {
        warnings.push("Zero tokens");
    }
    if msg.timestamp > Utc::now() + Duration::hours(1) {
        warnings.push("Future timestamp");
    }

    ValidationResult { valid: warnings.is_empty(), warnings }
}
```

- Validate all parsed messages before DB insertion
- Log warnings but don't reject (data loss is worse than imperfect data)
- Track validation stats in metadata table

### 6.3 Database Backup

```bash
aitop --backup  # Copies DB to ~/.local/share/aitop/backups/aitop-2026-03-19.db
```

- Automatic backup before migrations
- Keep last 5 backups, prune older
- Configurable backup dir

---

## Architecture Target (v1.0)

```
 ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
 │ Claude Code  │  │  Admin API   │  │  OTel/Other  │
 │   Provider   │  │   Provider   │  │   Provider   │
 └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
        │                 │                  │
        ▼                 ▼                  ▼
 ┌─────────────────────────────────────────────────┐
 │              Provider Registry                   │
 │         (scan, parse, watch, pricing)            │
 └───────────────────────┬─────────────────────────┘
                         │
                         ▼
 ┌─────────────────────────────────────────────────┐
 │              Database (SQLite WAL)              │
 │  ┌──────────┐  ┌──────────┐  ┌──────────────┐  │
 │  │ sessions │  │ messages │  │  file_index   │  │
 │  │ +provider│  │ +provider│  │  +provider    │  │
 │  └──────────┘  └──────────┘  └──────────────┘  │
 │  ┌──────────┐  ┌──────────┐                    │
 │  │ schema_v │  │ metadata │                    │
 │  └──────────┘  └──────────┘                    │
 └───────────────────────┬─────────────────────────┘
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
        ┌──────────┐ ┌────────┐ ┌──────────┐
        │Aggregator│ │Anomaly │ │ Exporter │
        │ (queries)│ │Detector│ │ (export) │
        └────┬─────┘ └────┬───┘ └────┬─────┘
             │            │          │
             ▼            ▼          ▼
        ┌──────────────────────────────────┐
        │         TUI / --light / --export │
        └──────────────────────────────────┘
```

---

## Key Design Decisions

### Why not a separate process for ingestion?

Keeping scanner/parser/watcher in-process avoids IPC complexity, socket management,
and daemon lifecycle. SQLite WAL mode gives us concurrent read/write within a single
process. The watcher thread is lightweight (~0.1% CPU idle). If we ever need a daemon
(e.g., for continuous alerting), we can extract it then.

### Why SQLite over embedded key-value stores?

SQL aggregations are the core of every view. Computing burn rates, daily trends, model
breakdowns, and anomaly detection as SQL queries is simpler and faster than scanning
key-value ranges. SQLite's query planner handles index selection automatically.

### Why provider trait over plugin system?

For v0.3, compiled-in providers are simpler, faster, and easier to test. The plugin
system (WASM) is a v1.0 goal that requires a stable API contract. Starting with a trait
lets us iterate on the API before freezing it for plugins.
