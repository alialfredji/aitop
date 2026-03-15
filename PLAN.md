# Plan: `aitop` — btop for AI

## Current State

Developers using Claude Code (and other AI coding agents) have no real-time, unified terminal dashboard for monitoring token spend, session activity, and burn rate. Existing tools (tokscale, ccusage, tu) are mostly static table reporters or basic TUI viewers — none deliver the btop experience: a live, information-dense, keyboard-driven dashboard you leave running in a tmux pane.

## Desired End State

A single Rust binary (`aitop`) that:
- Launches instantly, feels like btop — live-updating, responsive, keyboard-driven
- Parses local JSONL session files with zero auth required
- Optionally connects to Anthropic Admin API for org-level data
- Shows burn rate, cost trends, model breakdown, session drill-down
- Has btop-style highlighted-letter shortcuts, mouse support, themes
- Works on macOS and Linux, in any terminal ≥ 80x24

## Scope Boundaries

**In scope (v0.1):**
- Claude Code local JSONL parsing (primary data source)
- Live file watching (kqueue/inotify) for real-time updates
- 4-panel dashboard layout
- Keyboard navigation with highlighted shortcuts
- Config file for optional API key, theme, refresh rate

**Out of scope (v0.1):**
- OpenAI / Gemini / other provider support (future plugin system)
- Anthropic Admin API integration (v0.2 — requires org account)
- Web UI or image export
- Leaderboards or social features
- Package manager distribution (Homebrew, npm wrapper — post-launch)

---

## Stack

| Layer | Choice | Why |
|-------|--------|-----|
| Language | **Rust** | Single binary, fast JSONL parsing, matches btop's performance DNA |
| TUI framework | **Ratatui + Crossterm** | Best widget library (charts, sparklines, gauges, tables), cross-platform |
| File watching | **notify** crate | Cross-platform fs events (kqueue on macOS, inotify on Linux) |
| JSON parsing | **simd-json** or **serde_json** | Fast JSONL line parsing; simd-json for bulk initial scan |
| Local DB | **SQLite via rusqlite** | Incremental index of parsed sessions — instant startup after first run |
| Config | **TOML via toml** crate | `~/.config/aitop/config.toml` — standard, human-readable |
| Async runtime | **tokio** | File watcher events, optional API polling, timer ticks |

### Why SQLite?

Every competitor re-parses all JSONL files on every launch. On a machine with months of Claude Code history, that's slow. `aitop` indexes once into `~/.local/share/aitop/sessions.db`, then incrementally appends new data. Result: **instant startup** after first run, and we get SQL-powered aggregations for free (top sessions by cost, daily trends, model breakdowns — all just queries).

---

## UI Design

### Philosophy

Steal btop's DNA:
- **Highlighted shortcut letters** in panel titles (e.g., "**D**ashboard", "**S**essions", "**M**odels", "**T**rends") — the letter is rendered in accent color + underline
- **Information density** — no wasted space, every cell tells you something
- **Responsive** — adapts layout to terminal size (compact mode < 100 cols)
- **Live** — default 2s refresh tick, updates in-place
- **Mouse + keyboard** — click panel titles to switch, scroll tables

### Color Palette (Default "Ember" theme)

```
Background:  terminal default (transparent-friendly)
Primary:     #FF6B35 (warm orange — burn rate, accent)
Secondary:   #4ECDC4 (teal — input tokens, cache)
Tertiary:    #FFE66D (gold — output tokens, cost)
Muted:       #555555 (borders, labels)
Danger:      #FF1744 (over budget, high burn)
Success:     #00E676 (under budget, low burn)
```

Additional themes: `nord`, `dracula`, `gruvbox`, `mono`, `catppuccin`

### Layout (≥ 120 cols × 40 rows)

```
┌─ aitop ──────────────────────────────────────────────────────────────────────┐
│  [D]ashboard   [S]essions   [M]odels   [T]rends          [?]Help  [q]Quit  │
├──────────────────────────────────┬───────────────────────────────────────────┤
│  BURN RATE          SPEND TODAY  │  TOKEN FLOW (last 60 min)                │
│  $2.34/hr ▲         $18.72      │  ▁▂▃▅▇█▇▅▃▂▁▁▂▄▆█▇▅▃▂▁▁▁▂▃▅▇█▇▅▃▁    │
│                                  │  ── input  ── output  ── cache_read     │
│  Spend This Week    Budget       │                                         │
│  $142.30 / $200  ████████░░ 71%  │  CACHE HIT RATIO                       │
│                                  │  ████████████████░░░░ 82%               │
├──────────────────────────────────┼───────────────────────────────────────────┤
│  MODEL BREAKDOWN                 │  ACTIVE SESSIONS                        │
│                                  │                                         │
│  opus-4-6    ████████████ $312   │  #1 echopad    opus   $4.20  12m ago    │
│  haiku-4-5   ██           $10   │  #2 dotfiles   haiku  $0.31   2h ago    │
│  sonnet-4-6  █             $4   │  #3 personal   opus   $8.10   3h ago    │
│                                  │  #4 ccode      sonnet $1.02   5h ago    │
│  Total: $395.84 all-time         │                                         │
│  Today: opus 89% haiku 8% son 3%│  ↑↓ navigate  Enter drill-in           │
├──────────────────────────────────┴───────────────────────────────────────────┤
│  RECENT ACTIVITY                                                            │
│  14:23  echopad     opus-4-6     324 in / 1,204 out / 18k cache   $0.08   │
│  14:21  echopad     opus-4-6     289 in /   891 out / 18k cache   $0.06   │
│  14:18  dotfiles    haiku-4-5  1,024 in /   445 out /  4k cache   $0.01   │
│  14:15  echopad     opus-4-6     512 in / 2,301 out / 18k cache   $0.12   │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Compact Layout (< 100 cols)

Stacks panels vertically, hides sparkline legend, abbreviates model names.

### Panel Details

#### [D]ashboard (default view — shown above)
- **Top-left:** Hero metrics — burn rate ($/hr, with ▲▼ trend arrow), today's spend, weekly spend with budget gauge
- **Top-right:** Token flow sparkline (braille dots, 60 data points = last 60 min), cache hit ratio bar
- **Mid-left:** Model cost breakdown with proportional bars
- **Mid-right:** Active sessions list (project name, model, cost, recency)
- **Bottom:** Recent activity feed (last N API calls, scrollable)

#### [S]essions view
```
┌─ Sessions ──────────────────────────────────────────────────────────────────┐
│  / filter   ↑↓ navigate   c cost  t time  n tokens  Enter details         │
├────┬──────────────┬─────────────┬──────────┬──────────┬────────┬───────────┤
│  # │ Project      │ Model       │ Tokens   │ Cost     │ Msgs   │ When      │
├────┼──────────────┼─────────────┼──────────┼──────────┼────────┼───────────┤
│  1 │ echopad      │ opus-4-6    │  245,201 │  $42.30  │   124  │ 12m ago   │
│  2 │ personal     │ opus-4-6    │  189,332 │  $31.20  │    89  │  3h ago   │
│  3 │ dotfiles     │ haiku-4-5   │   34,021 │   $1.20  │    45  │  2h ago   │
│ .. │ ...          │ ...         │      ... │     ...  │   ...  │ ...       │
├────┴──────────────┴─────────────┴──────────┴──────────┴────────┴───────────┤
│ Total: 847 sessions   427K input   604K output   $395.84                   │
└─────────────────────────────────────────────────────────────────────────────┘
```
- Sortable by any column (press the highlighted letter)
- `/` to filter by project name
- `Enter` to drill into session detail (message-by-message token breakdown)
- `d` for date range filter

#### [M]odels view
```
┌─ Models ────────────────────────────────────────────────────────────────────┐
│                                                                             │
│  claude-opus-4-6                                              $382.02      │
│  ████████████████████████████████████████████████████████████  96.5%        │
│  249K input  ·  558K output  ·  82% cache hit  ·  4,201 calls             │
│                                                                             │
│  claude-haiku-4-5                                               $9.68      │
│  ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   2.4%      │
│  178K input  ·  42K output  ·  71% cache hit  ·  1,893 calls              │
│                                                                             │
│  claude-sonnet-4-6                                              $4.14      │
│  █░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   1.0%      │
│  247 input  ·  4.6K output  ·  0% cache hit  ·  12 calls                  │
│                                                                             │
│  $/token: opus $0.091  haiku $0.005  sonnet $0.345                         │
└─────────────────────────────────────────────────────────────────────────────┘
```
- Per-model deep stats: avg tokens/call, cache efficiency, cost/token
- Proportional bars showing relative spend

#### [T]rends view
```
┌─ Trends ────────────────────────────────────────────────────────────────────┐
│  Daily Spend (last 30 days)                                     Avg: $6.20 │
│  $20 ┤                                                                     │
│      │              ╭╮                                                     │
│  $15 ┤         ╭────╯╰╮    ╭╮                                             │
│      │    ╭╮  ╭╯      ╰╮  ╭╯╰╮       ╭╮                                  │
│  $10 ┤╭──╯╰──╯        ╰──╯   ╰╮  ╭──╯╰──╮                               │
│      ││                        ╰──╯      ╰──╮  ╭╮                        │
│   $5 ┤│                                      ╰──╯╰──╮                     │
│      ││                                              ╰──                   │
│   $0 ┤┼────────┼────────┼────────┼────────┼────────┼──                     │
│      Feb 14    Feb 21    Feb 28    Mar 7     Mar 14                         │
│                                                                             │
│  ── total  ── opus  ── haiku  ── sonnet       [w]eek [m]onth [a]ll        │
├─────────────────────────────────────────────────────────────────────────────┤
│  Projection: $186/mo at current rate   │   Peak hour: 2-3 PM ($4.20/hr)   │
│  This week vs last: +12% ($8.20 more)  │   Quietest: 6-7 AM ($0.10/hr)   │
└─────────────────────────────────────────────────────────────────────────────┘
```
- Line chart of daily spend (Ratatui `Chart` widget with braille markers)
- Projection based on trailing 7-day average
- Time-of-day heatmap for usage patterns
- Week-over-week comparison

---

## Keyboard Shortcuts

### Global (btop-style — highlighted letter in panel title)

| Key | Action |
|-----|--------|
| `d` | Switch to **D**ashboard |
| `s` | Switch to **S**essions |
| `m` | Switch to **M**odels |
| `t` | Switch to **T**rends |
| `q` / `Ctrl-C` | **Q**uit |
| `?` / `F1` | Toggle help overlay |
| `r` | Force **r**efresh |
| `Tab` | Cycle panels (within view) |
| `1-4` | Quick switch to view 1-4 |
| `/` | Open search/filter |
| `Esc` | Close overlay / clear filter |
| `+` / `-` | Increase / decrease refresh interval |

### In table views (Sessions)

| Key | Action |
|-----|--------|
| `↑` / `↓` / `j` / `k` | Navigate rows |
| `Enter` | Drill into selected item |
| `Backspace` | Back to list |
| `c` | Sort by **c**ost |
| `n` | Sort by toke**n**s |
| `p` | Sort by **p**roject |

### In Trends view

| Key | Action |
|-----|--------|
| `w` | Show last **w**eek |
| `m` | Show last **m**onth |
| `a` | Show **a**ll time |
| `←` / `→` | Pan time window |

---

## Architecture

```
src/
├── main.rs                 # Entry point, arg parsing, app loop
├── app.rs                  # App state machine, event dispatch
├── config.rs               # TOML config loading (~/.config/aitop/config.toml)
│
├── data/
│   ├── mod.rs
│   ├── scanner.rs          # Scan ~/.claude/projects/ for JSONL files
│   ├── parser.rs           # Parse JSONL lines → typed structs
│   ├── db.rs               # SQLite schema, upsert, queries
│   ├── watcher.rs          # File watcher (notify crate) → triggers re-parse
│   └── aggregator.rs       # Compute burn rate, trends, model breakdown
│
├── ui/
│   ├── mod.rs
│   ├── layout.rs           # Responsive layout (wide vs compact)
│   ├── theme.rs            # Color themes
│   ├── dashboard.rs        # Dashboard panel rendering
│   ├── sessions.rs         # Sessions table view
│   ├── models.rs           # Models breakdown view
│   ├── trends.rs           # Trends / charts view
│   ├── help.rs             # Help overlay
│   └── widgets/
│       ├── burn_rate.rs    # Hero metric widget (big number + trend arrow)
│       ├── sparkline.rs    # Token flow sparkline
│       ├── bar_chart.rs    # Model breakdown bars
│       └── activity.rs    # Recent activity feed
│
└── providers/              # Future: multi-provider support
    ├── mod.rs
    ├── claude_local.rs     # JSONL file parser (v0.1)
    └── claude_api.rs       # Admin API client (v0.2)
```

### Data Flow

```
JSONL files (fs watch)
        │
        ▼
   Scanner/Parser ──► SQLite DB ──► Aggregator ──► UI State ──► Ratatui render
        │                               ▲
     (first run:                  (every 2s tick:
      full scan)                  incremental query)
```

### SQLite Schema

```sql
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,  -- session UUID
    project     TEXT NOT NULL,     -- decoded project path
    started_at  TEXT NOT NULL,     -- ISO 8601
    updated_at  TEXT NOT NULL,
    model       TEXT,              -- primary model used
    version     TEXT               -- claude code version
);

CREATE TABLE messages (
    id              TEXT PRIMARY KEY,  -- message UUID
    session_id      TEXT NOT NULL REFERENCES sessions(id),
    type            TEXT NOT NULL,     -- 'user', 'assistant', 'tool_result'
    timestamp       TEXT NOT NULL,
    model           TEXT,
    input_tokens    INTEGER DEFAULT 0,
    output_tokens   INTEGER DEFAULT 0,
    cache_read      INTEGER DEFAULT 0,
    cache_creation  INTEGER DEFAULT 0,
    cost_usd        REAL DEFAULT 0.0
);

CREATE INDEX idx_messages_session ON messages(session_id);
CREATE INDEX idx_messages_timestamp ON messages(timestamp);
CREATE INDEX idx_messages_model ON messages(model);

-- Track which files we've already parsed and their last byte offset
CREATE TABLE file_index (
    path        TEXT PRIMARY KEY,
    last_offset INTEGER DEFAULT 0,
    last_mtime  TEXT
);
```

### Key Aggregation Queries

```sql
-- Burn rate (last hour)
SELECT SUM(cost_usd) / (julianday('now') - julianday(MIN(timestamp))) / 24
FROM messages WHERE timestamp > datetime('now', '-1 hour');

-- Today's spend
SELECT SUM(cost_usd) FROM messages
WHERE date(timestamp) = date('now');

-- Model breakdown
SELECT model, SUM(cost_usd), SUM(input_tokens), SUM(output_tokens),
       SUM(cache_read), SUM(cache_creation)
FROM messages GROUP BY model ORDER BY SUM(cost_usd) DESC;

-- Daily trend (last 30 days)
SELECT date(timestamp) as day, SUM(cost_usd)
FROM messages WHERE timestamp > datetime('now', '-30 days')
GROUP BY day ORDER BY day;

-- Sessions ranked by cost
SELECT s.id, s.project, s.model, s.started_at,
       SUM(m.cost_usd) as total_cost,
       SUM(m.input_tokens + m.output_tokens) as total_tokens,
       COUNT(*) as msg_count
FROM sessions s JOIN messages m ON s.id = m.session_id
GROUP BY s.id ORDER BY total_cost DESC;
```

---

## Cost Calculation

Since JSONL files contain raw token counts but not costs, we compute costs using hardcoded pricing (updated with releases):

```rust
// Pricing per million tokens (as of March 2026)
fn cost_per_mtok(model: &str) -> (f64, f64, f64, f64) {
    // (input, output, cache_read, cache_creation)
    match model {
        m if m.contains("opus") => (15.0, 75.0, 1.50, 18.75),
        m if m.contains("sonnet") => (3.0, 15.0, 0.30, 3.75),
        m if m.contains("haiku") => (0.80, 4.0, 0.08, 1.0),
        _ => (3.0, 15.0, 0.30, 3.75), // default to sonnet pricing
    }
}
```

Future: fetch live pricing from LiteLLM/OpenRouter (like tokscale does).

---

## Config File

`~/.config/aitop/config.toml`:

```toml
# Refresh interval in seconds (default: 2)
refresh = 2

# Color theme: ember, nord, dracula, gruvbox, mono, catppuccin
theme = "ember"

# Weekly budget (optional — enables budget gauge)
# weekly_budget = 200.0

# Claude Code session directory (auto-detected, override if needed)
# claude_data_dir = "~/.claude/projects"

# Anthropic Admin API key (optional — enables org-level data in v0.2)
# admin_api_key = "sk-ant-admin..."
```

---

## Implementation Steps

### Phase 1: Foundation (scaffold + data layer)

1. `cargo init aitop` with dependencies:
   - `ratatui`, `crossterm`, `tokio`, `serde`, `serde_json`, `rusqlite`, `notify`, `toml`, `dirs`, `chrono`, `clap`
2. Implement `config.rs` — load/create default config
3. Implement `scanner.rs` — walk `~/.claude/projects/`, find all `.jsonl` files
4. Implement `parser.rs` — parse JSONL lines into typed structs, extract usage fields
5. Implement `db.rs` — SQLite schema, bulk insert, incremental upsert via `file_index`
6. Implement `aggregator.rs` — burn rate, daily totals, model breakdown queries
7. **Checkpoint:** CLI mode that prints summary table (like `--light` mode) to verify data pipeline

### Phase 2: TUI Shell

8. Implement `app.rs` — App state (current view, selected row, scroll position), event loop (key events, tick events, fs events)
9. Implement `layout.rs` — responsive constraint-based layout, wide vs compact detection
10. Implement `theme.rs` — color palette structs, theme loading
11. Implement tab bar with highlighted shortcut letters
12. **Checkpoint:** Empty TUI shell with working tab switching, quit, help overlay

### Phase 3: Dashboard View

13. Implement `burn_rate.rs` widget — big number with trend arrow
14. Implement `sparkline.rs` — token flow (braille markers, last 60 data points)
15. Implement `bar_chart.rs` — model cost breakdown
16. Implement `activity.rs` — recent activity feed (scrollable)
17. Wire dashboard to aggregator queries
18. **Checkpoint:** Fully functional dashboard with real data

### Phase 4: Sessions + Models + Trends

19. Implement `sessions.rs` — sortable, filterable table, drill-in detail view
20. Implement `models.rs` — per-model stats with proportional bars
21. Implement `trends.rs` — daily spend chart (Ratatui `Chart`), projections, time-of-day stats
22. **Checkpoint:** All 4 views working with real data

### Phase 5: Live Mode + Polish

23. Implement `watcher.rs` — file watcher → incremental parse → DB update → UI refresh
24. Add mouse support (click tab titles, scroll tables)
25. Add all themes
26. Add `--light` flag for non-TUI table output
27. Add `clap` CLI args (`--theme`, `--refresh`, `--project`, `--since`, `--until`)
28. **Checkpoint:** Feature-complete v0.1

### Phase 6: Release

29. README with screenshots/GIF
30. Cargo.toml metadata, license (MIT)
31. Test on macOS + Linux (CI)
32. `cargo publish` + GitHub release

---

## Success Criteria

- [ ] `aitop` launches in < 200ms (after first indexing run)
- [ ] Dashboard shows accurate burn rate, matching `/cost` command and tokscale output
- [ ] Live updates appear within 3s of a new Claude Code API call
- [ ] All 4 views render correctly at 80x24 (compact) and 200x60 (wide)
- [ ] Keyboard shortcuts match the documented map, highlighted letters visible
- [ ] Zero network calls in default mode (local JSONL only)
- [ ] Single binary, no runtime dependencies
