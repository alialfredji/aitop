# Plan v3: aitop Feature Roadmap

## Current State (v0.2.0 — shipped)

All v0.2 features are complete: 4 polished views, 6 themes, SQLite-backed pipeline with
incremental indexing, file watcher with LIVE/IDLE indicator, session detail popup, filter
overlay, sparklines, heatmap, contribution calendar, delta banner, budget gauge, cost
gradients, and distribution (Homebrew, cargo, npx, GitHub releases, install script).

---

## v0.3 — Multi-Source & Intelligence

### Theme: "See everything, understand everything"

v0.3 transforms aitop from a Claude Code session viewer into a comprehensive AI cost
intelligence platform.

---

### Phase A: Multi-Provider Data Ingestion

**Goal:** Support token/cost data from providers beyond Claude Code local JSONL.

#### A1. Provider Adapter Trait

```rust
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn scan(&self) -> Result<Vec<SessionFile>>;
    fn parse(&self, path: &Path, offset: u64) -> Result<Vec<ParsedMessage>>;
    fn pricing(&self) -> &PricingTable;
}
```

- Extract current Claude Code logic into `ClaudeCodeProvider`
- Each provider implements scan → parse → pricing
- Provider registry in config: `[providers]` table

#### A2. Anthropic Admin API Provider

- `GET /v1/organizations/usage_report/messages` (1h/1d buckets)
- `GET /v1/organizations/cost_report` (daily, USD cents)
- Requires `admin_api_key` in config (already reserved)
- Poll interval: 60s (API rate limit: 1/min)
- Merge API-level data with local JSONL (dedup by session ID where possible)
- New dashboard section: "Org Usage" showing team-wide stats

#### A3. OpenAI Usage Provider (stretch)

- Read from `~/.openai/` or OpenAI API usage endpoint
- Parse ChatGPT/API usage data
- Unified cost model with provider-tagged sessions

#### A4. Claude Code OpenTelemetry Provider

- Parse OTLP exports when `CLAUDE_CODE_ENABLE_TELEMETRY=1`
- Support Prometheus scrape format
- Map `claude_code.token.usage` / `claude_code.cost.usage` metrics

---

### Phase B: Cost Intelligence

**Goal:** Move from "what happened" to "what does it mean."

#### B1. Cost Anomaly Detection

- Compute rolling 7-day mean and std deviation per project
- Flag sessions/days exceeding 2σ threshold
- Visual indicator in sessions table: `⚠` prefix on anomalous rows
- Dashboard widget: "Anomalies" panel listing flagged items
- Config: `anomaly_threshold = 2.0` (standard deviations)

#### B2. Burn Rate Forecasting

- Linear regression on daily spend (7d/30d windows)
- Project monthly spend at current rate
- Dashboard: "At this rate: $X by end of week/month"
- Budget gauge: projected overshoot date if trending over budget
- Visual: dashed forecast line on daily spend chart

#### B3. Cost Optimization Suggestions

- Detect sessions with low cache hit ratios
- Identify opus usage where haiku/sonnet might suffice (high token count, low complexity signals)
- Dashboard tip: "Switch echopad to sonnet to save ~$X/week"
- Based on model mix and token patterns, not message content

#### B4. Session Comparison

- Select two sessions (mark with `x` in sessions table)
- Side-by-side popup showing: cost, tokens, model, duration, cache ratio
- Useful for comparing before/after optimization attempts

---

### Phase C: Export & Reporting

**Goal:** Get data out of the terminal.

#### C1. JSON Export

- `aitop --export json > report.json`
- Full data dump: sessions, models, daily spend, anomalies
- Structured for programmatic consumption

#### C2. Markdown Summary Report

- `aitop --export markdown > report.md`
- Human-readable report with tables and stats
- Suitable for Slack/email/PR descriptions
- Template:
  ```
  # AI Usage Report — Week of Mar 17, 2026

  **Total Spend:** $142.30 | **Sessions:** 23 | **Top Model:** opus-4-6 (68%)

  ## Daily Breakdown
  | Date | Spend | Sessions | Top Project |
  ...

  ## Anomalies
  - Mar 14: $28.40 (3.2x average) — echopad
  ```

#### C3. CSV Export

- `aitop --export csv > sessions.csv`
- Flat session-level data for spreadsheet analysis

#### C4. Clipboard Copy

- `y` key in sessions table copies selected row data
- `Y` copies full session detail as markdown
- Status bar flash: "Copied to clipboard"

---

### Phase D: Advanced UI

**Goal:** Match btop's information density and interaction depth.

#### D1. Resizable Panels

- Drag panel borders with mouse (or `+`/`-` keys)
- Remember panel sizes per view
- Persist in `~/.config/aitop/layout.toml`

#### D2. Projects View (5th tab)

- New view: `p` / `5` (move theme cycling to `T` or `F2`)
- Project-centric: list projects with total cost, session count, last active
- Drill into project → show project's sessions
- Project cost bars (already computed, needs its own view)

#### D3. Mouse Support

- Click to select table rows
- Click tab bar to switch views
- Scroll wheel for table navigation
- Click popup close button
- Toggle via config: `mouse = true`

#### D4. Notifications

- Desktop notification when budget threshold crossed (75%, 90%, 100%)
- Uses `notify-rust` crate for cross-platform notifications
- Config: `notifications = true`, `notification_thresholds = [75, 90, 100]`

#### D5. Custom Dashboard Layout

- Config-driven dashboard widget arrangement
- Users choose which panels appear and their positions
- `[dashboard.widgets]` table in config

---

### Phase E: Performance & Reliability

#### E1. Startup Progress Bar

- Show indexing progress during first run with large histories
- Animated: `Indexing sessions... ████████░░░░ 67% (142/212 files)`
- Skip if indexing completes in < 500ms

#### E2. Database Migrations

- Schema version table
- Automatic migration on version upgrade
- Backward-compatible: old versions can read new DB (ignore new columns)

#### E3. Concurrent Parsing

- Rayon-based parallel JSONL parsing during initial scan
- Split file list across cores
- Target: 4x speedup on initial index of large histories

#### E4. Memory-Mapped File Reading

- Use `memmap2` for large JSONL files
- Avoid loading entire files into memory
- Parse directly from memory-mapped regions

---

## Implementation Priority

```
High Impact, Low Effort (do first):
├── B1: Cost anomaly detection
├── C1: JSON export
├── C2: Markdown summary
└── D2: Projects view

High Impact, High Effort (plan carefully):
├── A1-A2: Provider trait + Admin API
├── B2: Burn rate forecasting
└── D3: Mouse support

Low Impact, Low Effort (quick wins):
├── C3: CSV export
├── C4: Clipboard copy
└── E1: Startup progress bar

Low Impact, High Effort (defer):
├── A3: OpenAI provider
├── D1: Resizable panels
├── D5: Custom dashboard layout
└── E4: Memory-mapped files
```

---

## Config Evolution

```toml
# ~/.config/aitop/config.toml (v0.3)

refresh = 2
theme = "ember"
weekly_budget = 200.0
mouse = false
notifications = false
notification_thresholds = [75, 90, 100]
anomaly_threshold = 2.0  # standard deviations

[providers.claude_code]
enabled = true
data_dir = "~/.claude/projects"

[providers.admin_api]
enabled = false
api_key = "sk-ant-admin-..."
org_id = "org-..."
poll_interval = 60

[export]
default_format = "json"  # json, markdown, csv
```

---

## Breaking Changes

None planned. v0.3 is additive — all v0.2 behavior preserved. New features are opt-in
via config or CLI flags.

---

## Success Metrics

- **Coverage:** ≥2 data sources supported (local JSONL + Admin API)
- **Intelligence:** Anomaly detection catches 80%+ of spend spikes
- **Performance:** <200ms startup on indexed DB, <2s initial index for 200 files
- **Export:** Users can generate reports without leaving terminal
- **Adoption:** Projects view becomes most-used after Dashboard
