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

## v0.2 — Polish (in progress)

### Phase A: Core Polish
- [ ] Persistent status bar with context-sensitive hints
- [ ] Sort indicators (▲▼) in table column headers
- [ ] Theme cycling with `p` key
- [ ] Cost color gradients (green → yellow → orange → red)
- [ ] Budget gauge widget with color thresholds
- [ ] Compact layout fixes for 80x24 terminals

### Phase B: Interaction Depth
- [ ] Session detail popup (Enter to drill in, Esc to close)
- [ ] Filter/search overlay (`/` key) with incremental matching
- [ ] Inline sparklines (▁▂▃▅▇) in session table rows
- [ ] Sort direction toggle (ascending/descending)

### Phase C: Creative Features
- [ ] "Since last check" delta banner on startup
- [ ] Time-of-day heatmap (7×24 grid in Trends)
- [ ] GitHub-style contribution calendar
- [ ] Project cost attribution breakdown
- [ ] Token efficiency score + cache savings metric

### Phase D: Live Mode
- [ ] File watcher → incremental DB → real-time UI updates
- [ ] Live indicator (● LIVE / ○ IDLE)
- [ ] Event-driven refresh with debounce
- [ ] Startup progress bar for initial indexing

## v0.3 — Future

- [ ] Multi-provider support (OpenAI, Gemini) via plugin/adapter pattern
- [ ] Anthropic Admin API integration (org-level data)
- [ ] Cost anomaly detection (z-score flagging)
- [ ] Session comparison (side-by-side)
- [ ] Export (JSON, markdown summary)
- [ ] Distribution: Homebrew, cargo publish, GitHub releases
- [ ] Machine-readable output (`aitop --json`) for multi-agent workflow observability
  - Expose budget remaining, burn rate, per-model spend, time until provider reset
  - Agents/orchestrators query aitop and self-throttle or switch models
  - aitop stays pure observability — enforcement is external (Unix philosophy)
