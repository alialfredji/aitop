# Plan v2: aitop Polish & Feature Roadmap

## Current State (v0.1 — shipped)

Working TUI with 4 views, SQLite-backed data pipeline, 6 themes, btop-style keyboard shortcuts. Functional but rough: layout needs polish, no animations, limited interactivity, missing several high-impact features visible in competitors.

### Key fix shipped: d/s/m/t are global nav keys

Fixed a key conflict bug where `m` was overloaded: it was both the global "switch to Models" key and the Trends view "month" range key. Resolution: `d`/`s`/`m`/`t` are now always global navigation keys handled first in `handle_key()`, and view-specific keys use non-conflicting bindings. Trends uses `o` for month (m**o**nth) and `←`/`→` to cycle. Sessions uses `u` for updated/recent sort.

## Goals

1. **Visual polish** — match btop's density and feel, surpass tokscale's aesthetics
2. **Interaction depth** — drill-down, filtering, sorting, popup panels
3. **Creative features** — things nobody else has done
4. **Robustness** — handle edge cases, large datasets, live updates

---

## UX Improvements (Priority Order)

### 1. Status Bar (bottom)

Replace empty space with a persistent bottom status bar showing context-sensitive hints and live stats.

```
─────────────────────────────────────────────────────────────────
 aitop v0.1.0 │ 85 sessions │ $1,905 all-time │ ?:help q:quit │ ⟳ 2s
```

- Left: version + session count + all-time spend
- Right: available shortcuts for current view + refresh indicator
- Changes per-view (Sessions view shows sort mode, Trends shows range)

### 2. Inline Sparklines in Tables

Add tiny `▁▂▃▅▇` sparklines inside table cells showing 7-day trend per session/model.

```
│ echopad     │ opus-4-6    │ ▁▂▃▅▇▅▃ │ $42.30 │
│ dotfiles    │ haiku-4-5   │ ▃▃▂▁▁▁▁ │  $1.20 │
```

This gives at-a-glance trend info without navigating to the Trends view.

### 3. Session Detail Popup

`Enter` on a session in Sessions view opens a popup overlay showing:

```
┌─ Session: echopad ──────────────────────────────────────────┐
│  Model: opus-4-6   Duration: 2h 14m   Messages: 124        │
│  Cost: $42.30      Input: 89K    Output: 156K   Cache: 94%  │
│                                                              │
│  Message Timeline                                            │
│  14:23 ██████████████████████ $0.42  (2.1K in / 8.4K out)   │
│  14:21 ████████████████      $0.31  (1.8K in / 6.1K out)    │
│  14:18 ████████              $0.12  (0.9K in / 3.2K out)    │
│  14:15 ██████████████████████████ $0.58 (3.1K in / 11K out) │
│  ...                                                         │
│                                                              │
│  Token Distribution          Cost Over Time                  │
│  input  ███████░░ 36%        ▁▂▃▅▇█▇▅▃▂▁                   │
│  output ██████████████ 64%                                   │
│  cache  ████████████████ 94%                                 │
│                                                              │
│  ↑↓ scroll   Esc close                                      │
└──────────────────────────────────────────────────────────────┘
```

### 4. Budget Gauge Widget

When `weekly_budget` is set in config, show a proper gauge bar with color thresholds:

```
  Budget  ████████████░░░░░░░░ 71% ($142 / $200)
          green < 60%    yellow 60-85%    red > 85%
```

### 5. Time-of-Day Heatmap (Trends view)

A 7×24 grid showing when you use AI the most:

```
  Usage Heatmap (last 7 days)
       00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23
  Mon  ░░ ░░ ░░ ░░ ░░ ░░ ░░ ▒▒ ▓▓ ██ ██ ██ ▓▓ ██ ██ ██ ▓▓ ▒▒ ░░ ░░ ░░ ░░ ░░ ░░
  Tue  ░░ ░░ ░░ ░░ ░░ ░░ ░░ ░░ ▓▓ ██ ██ ▓▓ ▒▒ ▓▓ ██ ██ ██ ▓▓ ▒▒ ░░ ░░ ░░ ░░ ░░
  ...
```

Color-mapped: `░` = $0-1, `▒` = $1-5, `▓` = $5-10, `█` = $10+

### 6. GitHub-Style Contribution Calendar (Trends view)

Inspired by tokscale's contribution graph but in pure terminal:

```
  Contribution Graph (last 12 weeks)
  ░░░▒▓░░░▒▒▓▓░░░▓██▓░░▒▓██▓▒░▒▓████▒░▒▓██▓▒░▒▓████▒░░▒▓██▓░░▒▒▓▓░░
  Mon ░  ▒  ▒  ▓  ▒  ░  ▓  █  ▒  ░  ▓  █  ▓  ▒  ░  ░
  Wed ░  ▓  ▓  █  ▓  ▒  █  █  ▓  ▒  █  █  █  ▓  ▒  ░
  Fri ░  ▒  ▓  ▓  ▒  ░  ▓  █  ▒  ░  ▓  █  ▓  ▒  ░  ░
```

### 7. Smart Color Gradients for Cost

Instead of flat colors for cost numbers, use gradient mapping:

- `$0.01` → dim green
- `$1.00` → yellow
- `$10.00` → orange
- `$50.00+` → bright red

Applied everywhere costs appear (tables, activity feed, model bars).

### 8. Filter/Search Overlay

`/` opens a search overlay that filters across views:

```
┌─ Filter ──────────────────────────────┐
│ > echopad                             │
│                                       │
│  Sessions matching: 12                │
│  echopad (opus) — $42.30              │
│  echopad (haiku) — $1.20              │
│  echopad-deploy (sonnet) — $0.50      │
└───────────────────────────────────────┘
```

Incremental filtering with instant results.

### 9. Theme Cycling

Press `p` to cycle through themes live (like tokscale). Show theme name briefly in status bar.

### 10. Sort Indicator in Column Headers

Show which column is currently sorted and direction:

```
│ Project ▼    │ Model      │ Tokens     │ Cost ↕    │
```

`▼` = sorted descending, `▲` = ascending, `↕` = sortable

---

## Creative Features (Differentiators)

### A. "Since Last Check" Delta Mode

Store the last-viewed timestamp. On next launch, show a delta banner:

```
┌─ Since you last checked (2h ago) ─────────────────────────┐
│  +$8.34 spent  │  3 new sessions  │  opus ↑42%  haiku ↓18% │
└────────────────────────────────────────────────────────────┘
```

This is the "what did I miss?" moment — nobody does this.

### B. Cost Anomaly Detection

Flag sessions or days that are significantly above average:

```
  ⚠ Mar 14: $28.40 (3.2x your daily average of $8.90)
```

Simple z-score or IQR-based detection.

### C. Token Efficiency Score

Compute a "bang for your buck" metric:

```
  Efficiency: 2,847 tokens/$ (↑12% from last week)
  Cache savings: $142 saved by prompt caching
```

### D. Project-Level Cost Attribution

Group and filter by project:

```
  echopad       ████████████████████████ $342  (68%)
  personal-site ██████                   $89   (18%)
  dotfiles      ███                      $42   ( 8%)
  other         ██                       $32   ( 6%)
```

### E. Live File Watcher Integration

Wire up the existing `notify` watcher code to update the dashboard in real-time as Claude Code generates tokens. Show a "live" indicator when active sessions are detected.

```
  ● LIVE  echopad — opus-4-6 — $0.42 this minute
```

---

## Technical Improvements

### 1. Fix haiku token detection

Current parser misses some haiku sessions — the model field structure varies. Need to handle all model name formats.

### 2. Responsive layout improvements

- Test and fix compact mode (< 100 cols)
- Handle very small terminals (80×24) gracefully
- Handle very large terminals (200×60) by expanding panels

### 3. Incremental DB updates

The file watcher should trigger incremental ingestion (only parse new bytes from changed files), not re-parse everything.

### 4. Error resilience

- Handle malformed JSONL lines gracefully (skip, don't crash)
- Handle missing/locked session files
- Handle concurrent writes from active Claude Code sessions

### 5. Startup performance

- Measure and optimize first-run indexing time
- Show progress bar during initial scan if > 1s
- Target: < 200ms startup after first run

---

## Implementation Phases

### Phase A: Core Polish (immediate)
1. Status bar (persistent bottom bar with context hints)
2. Sort indicators in column headers
3. Theme cycling with `p` key
4. Color gradient for cost numbers
5. Budget gauge widget
6. Fix layout edge cases

### Phase B: Interaction Depth
7. Session detail popup (Enter to drill in, Esc to close)
8. Filter/search overlay (`/` key)
9. Inline sparklines in session table
10. Sort direction toggle (ascending/descending)

### Phase C: Creative Features
11. "Since last check" delta banner
12. Time-of-day heatmap
13. Contribution calendar
14. Project cost attribution view
15. Token efficiency score

### Phase D: Live Mode
16. Wire file watcher to incremental DB updates
17. Live indicator for active sessions
18. Auto-refresh on file change events

### Phase E: Distribution & Install
19. **Homebrew tap** — create `homebrew-aitop` tap repo with a Formula that builds from source (`cargo install`)
20. **GitHub Releases** — CI workflow to build release binaries for macOS (arm64, x86_64) and Linux (x86_64, aarch64), attach to GitHub releases
21. **cargo install** — publish to crates.io so `cargo install aitop` works
22. **npx wrapper** — npm package `aitop` that downloads the correct prebuilt binary for the platform (similar to how `@anthropic-ai/claude-code` distributes). `npx aitop` just works
23. **Shell one-liner** — `curl -fsSL https://raw.githubusercontent.com/.../install.sh | sh` script that detects OS/arch and downloads the right binary

Install priority: Homebrew > cargo install > GitHub releases > npx > shell script

### Phase F: Advanced
24. Cost anomaly detection
25. Session comparison (side-by-side)
26. Export (JSON, markdown summary)
27. `--watch` mode (stay running, update continuously)

---

## Scope Boundaries

**In scope:** All of phases A-E above.

**Out of scope (future):**
- Multi-provider support (OpenAI, Gemini) — design for it, don't build it yet
- Anthropic Admin API integration — needs org account, defer
- Social features / leaderboards — not our vibe
- Web dashboard — terminal-first
- Wrapped/year-in-review images — nice-to-have, not core

---

## Design Decisions

### Why d/s/m/t are global navigation keys

The four view-switching keys (`d` for Dashboard, `s` for Sessions, `m` for Models, `t` for Trends) are handled at the top level of `handle_key()` before dispatching to view-specific handlers. This prevents key conflicts: if `m` were handled in Trends view for "month", pressing `m` would set the month range instead of switching to Models. Making navigation keys unconditionally global means users can always switch views from anywhere without thinking about context. View-specific bindings must use alternative letters (e.g., `o` for month in Trends, `u` for updated sort in Sessions).

### Why SQLite (vs re-parsing JSONL every time)

Every competitor (tokscale, ccusage, tu) re-parses all JSONL files on every launch. With months of Claude Code history, that can mean scanning hundreds of megabytes of JSON on startup. SQLite gives us:
- **Instant subsequent startups** — the `file_index` table tracks byte offsets per file, so only new data is parsed
- **Free aggregations** — burn rate, daily trends, model breakdowns are SQL queries with indexes, not in-memory scans
- **Bounded memory** — large histories don't blow up RAM; SQLite pages on demand
- **Incremental updates** — the file watcher can append new records without re-indexing

### Why Rust + Ratatui

- **Performance:** btop set the bar. A TUI monitoring tool must feel instant. Rust's zero-cost abstractions and lack of GC mean smooth 60fps rendering and sub-200ms startup
- **Single binary:** `cargo build --release` produces a ~3MB static binary with no runtime dependencies. No Python venv, no Node modules, no system libraries
- **Widget ecosystem:** Ratatui provides Charts, Tables, Gauges, Sparklines, and Tabs out of the box. The `crossterm` backend handles terminal compatibility across macOS and Linux
- **Competitor alignment:** tokscale, toktop, and tu are all Rust + Ratatui — this is the established stack for terminal dashboards

### Why no auth required by default

Claude Code stores session data as JSONL files in `~/.claude/projects/`. These are local files readable by the current user — no API key, no OAuth flow, no network call needed. This means:
- **Zero friction:** `cargo install && aitop` works immediately
- **Zero token consumption:** monitoring your AI usage doesn't itself consume AI tokens
- **Privacy:** no data leaves the machine; aitop makes zero network calls
- The optional `admin_api_key` config field is reserved for future org-level data via the Anthropic Admin API, but it's never required
