# Architecture

## Stack

| Layer | Choice | Why |
|-------|--------|-----|
| Language | Rust | Single binary, fast JSONL parsing, btop-level performance |
| TUI | Ratatui + Crossterm | Best widget library, cross-platform terminal |
| File watching | notify | kqueue (macOS) / inotify (Linux) |
| JSON parsing | serde_json | Reliable, fast enough for line-by-line |
| Local DB | SQLite (rusqlite) | Incremental index → instant startup after first run |
| Config | TOML | `~/.config/aitop/config.toml` |
| Async | tokio | File watcher events, timer ticks |

## Why SQLite?

Competitors re-parse all JSONL files on every launch. We index once into `~/.local/share/aitop/sessions.db`, then incrementally append. SQL-powered aggregations (burn rate, daily trends, model breakdown) come for free.

## Data Flow

```
JSONL files (fs watch)
        │
        ▼
   Scanner/Parser ──► SQLite DB ──► Aggregator ──► UI State ──► Ratatui render
        │                               ▲
     (first run:                  (every 2s tick or
      full scan)                  on file change event)
```

## Source Layout

```
src/
├── main.rs              # Entry, arg parsing, event loop
├── app.rs               # App state machine, view enum, key dispatch
├── config.rs            # TOML config loading
├── data/
│   ├── scanner.rs       # Walk ~/.claude/projects/ for JSONL files
│   ├── parser.rs        # Parse JSONL lines → typed structs + cost calc
│   ├── db.rs            # SQLite schema, incremental upsert
│   ├── watcher.rs       # File watcher → triggers re-parse
│   └── aggregator.rs    # SQL queries → dashboard stats, model breakdown, etc.
└── ui/
    ├── layout.rs        # Responsive layout (wide vs compact)
    ├── theme.rs         # 6 color themes
    ├── dashboard.rs     # Dashboard panel rendering
    ├── sessions.rs      # Sessions table view
    ├── models.rs        # Models breakdown view
    ├── trends.rs        # Trends / charts view
    └── help.rs          # Help overlay
```

## SQLite Schema

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY, project TEXT, started_at TEXT, updated_at TEXT, model TEXT, version TEXT
);
CREATE TABLE messages (
    id TEXT PRIMARY KEY, session_id TEXT, type TEXT, timestamp TEXT, model TEXT,
    input_tokens INTEGER, output_tokens INTEGER, cache_read INTEGER, cache_creation INTEGER, cost_usd REAL
);
CREATE TABLE file_index (
    path TEXT PRIMARY KEY, last_offset INTEGER, last_mtime TEXT
);
```

## Design Decisions

### Global nav keys (d/s/m/t)
View-switching keys are handled at the top of `handle_key()` before view-specific handlers. This prevents conflicts — e.g., `d` always goes to Dashboard, never ambiguously sorts by date.

### No auth required by default
Primary data source is local JSONL files that Claude Code already writes. Zero network calls, zero token consumption, zero setup friction.

### Cost calculation
JSONL files have raw token counts but no costs. We compute using hardcoded Anthropic pricing (updated with releases). Future: fetch live pricing from LiteLLM/OpenRouter.
