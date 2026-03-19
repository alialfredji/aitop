# Roadmap

## v0.1 — Foundation (shipped)

- [x] Rust + Ratatui TUI with 4 views (Dashboard, Sessions, Models, Trends)
- [x] SQLite-backed data pipeline with incremental indexing
- [x] Claude Code local JSONL parsing (zero auth)
- [x] 6 color themes (ember, nord, dracula, gruvbox, catppuccin, mono)
- [x] btop-style highlighted shortcut letters
- [x] `--light` mode for non-interactive table output
- [x] Global nav keys (d/s/m/t) work from any view
- [x] 3MB single binary, zero runtime dependencies

## v0.2 — Polish (shipped)

### Phase A: Core Polish
- [x] Persistent status bar with context-sensitive hints
- [x] Sort indicators (▲▼) in table column headers
- [x] Theme cycling with `p` key
- [x] Cost color gradients (green → yellow → orange → red)
- [x] Budget gauge widget with color thresholds
- [x] Compact layout fixes for 80x24 terminals

### Phase B: Interaction Depth
- [x] Session detail popup (Enter to drill in, Esc to close)
- [x] Filter/search overlay (`/` key) with incremental matching
- [x] Inline sparklines (▁▂▃▅▇) in session table rows
- [x] Sort direction toggle (ascending/descending)

### Phase C: Creative Features
- [x] "Since last check" delta banner on startup
- [x] Time-of-day heatmap (7×24 grid in Trends)
- [x] GitHub-style contribution calendar
- [x] Project cost attribution breakdown
- [x] Token efficiency score + cache savings metric

### Phase D: Live Mode
- [x] File watcher → incremental DB → real-time UI updates
- [x] Live indicator (● LIVE / ○ IDLE)
- [x] Event-driven refresh with debounce
- [ ] Startup progress bar for initial indexing

### Phase E: Distribution (shipped)
- [x] Homebrew tap
- [x] GitHub releases with CI
- [x] cargo install
- [x] npx wrapper
- [x] Shell one-liner installer

## v0.3 — Multi-Source & Intelligence

### Phase A: Provider Abstraction
- [ ] Provider trait (scan, parse, pricing, watch)
- [ ] Refactor Claude Code logic into `ClaudeCodeProvider`
- [ ] Dynamic pricing config (`pricing.toml`)
- [ ] Database schema migrations system

### Phase B: Admin API Integration
- [ ] Anthropic Admin API provider (`/v1/organizations/usage_report`)
- [ ] Org-level cost data alongside local JSONL
- [ ] Deduplication between local + API data

### Phase C: Cost Intelligence
- [ ] Cost anomaly detection (z-score flagging, ⚠ indicators)
- [ ] Burn rate forecasting (linear regression, projected spend)
- [ ] Cost optimization suggestions (model mix analysis)
- [ ] Session comparison (side-by-side popup)

### Phase D: Export & Reporting
- [ ] JSON export (`--export json`)
- [ ] Markdown summary report (`--export markdown`)
- [ ] CSV export (`--export csv`)
- [ ] Clipboard copy from tables (`y` key)
- [ ] Export filters (`--project`, `--since`, `--model`)

### Phase E: Projects View
- [ ] 5th tab: Projects view (`p` / `5`)
- [ ] Project-centric navigation with drill-down
- [ ] Breadcrumb navigation (Projects > Project > Session)
- [ ] Per-project budget tracking

### Phase F: UX Enhancements
- [ ] Mouse support (click, scroll, hover)
- [ ] Multi-select in tables (compare, export selection)
- [ ] Desktop notifications for budget thresholds
- [ ] Startup progress bar for initial indexing

## v0.4 — Visual Richness

- [ ] Animated view transitions (slide, fade)
- [ ] Braille-resolution charts (2x density)
- [ ] Dual-axis charts (cost + tokens)
- [ ] Three-tier responsive layout (compact/standard/wide)
- [ ] Live streaming indicators with pulse animation
- [ ] Parallel JSONL parsing (rayon)
- [ ] Prepared statement cache + query result caching

## v0.5 — Terminal-Native Innovations

- [ ] Inline images via Sixel/Kitty protocol (with fallback)
- [ ] Split panes (view two tabs simultaneously)
- [ ] Session replay (time-lapse token activity)
- [ ] Tmux status bar integration (`--tmux-status`)

## v1.0 — Platform

- [ ] Team dashboard (Admin API)
- [ ] AI-powered usage insights (opt-in Claude API call)
- [ ] Plugin system (WASM-based community extensions)
- [ ] Multi-provider support (OpenAI, Gemini)
- [ ] Budget controls / governance
