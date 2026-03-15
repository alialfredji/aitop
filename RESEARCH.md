# Research: AI Usage TUI ("aitop" - working name)

## 1. Problem Statement

Monitor AI model token usage, billing, and rate limits from the terminal in a btop/htop-style live TUI. Start with Claude models, expand to other providers later.

## 2. Existing Tools (Competitive Landscape)

### Direct Competitors

| Tool | Stack | Approach | Strengths | Gaps |
|------|-------|----------|-----------|------|
| [tokscale](https://github.com/junhoyeo/tokscale) | Rust+Node, Ratatui | Reads local JSONL files from 16+ AI coding agents | Most comprehensive agent coverage, 4 TUI views, leaderboard, 3D graphs | Focused on coding agents only, no API-level org usage, no rate limit tracking |
| [toktop](https://github.com/htin1/toktop) | Rust | Calls OpenAI/Anthropic usage APIs | True API-level usage, multi-provider side-by-side | Requires Admin API key, no local session tracking, limited UI |
| [tokenusage (tu)](https://lib.rs/crates/tokenusage) | Rust, Ratatui+iced | Reads local JSONL files | Blazing fast (0.08s), TUI+GUI+image cards, activity inference | Local files only, no API usage data |
| [ccusage](https://github.com/ryoppippi/ccusage) | TypeScript | Reads local JSONL files | 5-hour billing window tracking, clean tables, MCP integration | Claude Code only, table output not interactive TUI |
| [Claude-Code-Usage-Monitor](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor) | Python | Reads local JSONL files | Burn rate predictions, depletion estimates | Single-tool, no multi-provider |

### Key Observation

**No existing tool combines:**
- Real-time btop-style interactive dashboard
- API-level org usage + local session data in unified view
- Rate limit / session limit tracking (the "Resets in 37 min / 24% used" from Claude Console)
- Multi-provider in a single pane (Claude + OpenAI + Gemini APIs)

This is the differentiation opportunity.

## 3. Data Sources Available

### 3a. Anthropic Admin API (Org users only)

**Usage endpoint:** `GET /v1/organizations/usage_report/messages`
- Requires Admin API key (`sk-ant-admin...`) - only org admins can provision
- Bucket widths: `1m` (up to 1440 buckets), `1h` (up to 168), `1d` (up to 31)
- Token types: uncached input, cached input, cache creation, output
- Group by: model, API key, workspace, service tier, context window, data residency, speed
- Polling: once per minute sustained; data freshness ~5 minutes

**Cost endpoint:** `GET /v1/organizations/cost_report`
- Daily granularity only (`1d`)
- Costs in USD cents (decimal strings)
- Tracks: token usage, web search, code execution costs
- Group by: workspace, description (parsed into model + geo)

**Auth:** `x-api-key` header with admin key + `anthropic-version: 2023-06-01`

### 3b. Local JSONL Session Files (Claude Code users - no auth needed)

**Location:** `~/.claude/projects/<url-encoded-project-path>/<session-uuid>.jsonl`

**Format:** Chain of typed message records linked by `parentUuid`:
- User prompts, assistant responses (text, tool calls, thinking), tool results
- Every assistant turn has a `usage` block with per-turn token counts
- Model selection, working directory, git state snapshots

**Known issue:** Output tokens may be undercounted (only intermediate streaming chunks recorded, not final count)

### 3c. Claude Code OpenTelemetry (Opt-in)

Metrics exported: `claude_code.token.usage`, `claude_code.cost.usage`, `claude_code.session.count`, `claude_code.lines_of_code.count`, etc.
- Set `CLAUDE_CODE_ENABLE_TELEMETRY=1` to enable
- Exporters: OTLP, Prometheus, console
- Rich attributes: session.id, model, token type, user.account_uuid

### 3d. Claude Console Web UI (Screenshot data)

The data shown on the Claude Console Settings > Usage page:
- **Plan usage limits:** Current session % used, reset countdown
- **Weekly limits:** All models % used, per-model (e.g., Sonnet only) % used, reset time
- **Extra usage toggle**

**Critical finding: This session/weekly limit data does NOT appear to have a public API.** It's only visible in the web UI. Scraping would be fragile and likely against ToS.

### 3e. OpenAI Usage API

- `GET /v1/organization/usage/completions` - requires org admin
- Dashboard available at platform.openai.com/usage

### 3f. `/cost` command in Claude Code

- Shows current session: total cost, API duration, wall duration, code changes
- Available to all Claude Code users (API users)
- Not available to Claude Max/Pro subscribers (they use `/stats`)

## 4. Authentication Options (Goal: no-token-consuming login)

| Method | Token Cost | Feasibility |
|--------|-----------|-------------|
| Admin API key (env var / config file) | Zero | Best for org users - just paste key once |
| Standard API key (env var) | Zero | Works for basic ops, but usage API needs admin key |
| Local JSONL parsing | Zero | No auth needed at all, but Claude Code only |
| OAuth browser flow | Zero tokens, but complex | Claude Code uses this; could potentially reuse session |
| Scraping Claude Console | Zero | Fragile, likely ToS violation - NOT recommended |

**Recommendation:** Start with local JSONL parsing (zero auth) + optional Admin API key for org-level data. This keeps the "no token consumption" requirement.

## 5. Technical Landscape for TUI Development

### Rust + Ratatui (Recommended)
- Industry standard for terminal UIs (btop, tokscale, tokenusage all use it)
- Crossterm backend for cross-platform terminal handling
- Rich widget library: charts, gauges, tables, sparklines, tabs
- High performance, single binary distribution

### Alternatives considered
- **Go + Bubbletea/Lipgloss:** Good ecosystem, but Ratatui has more widget variety
- **Python + Textual/Rich:** Slower startup, dependency management overhead
- **Node + Blessed/Ink:** Performance concerns for real-time updates

## 6. Key Technical Decisions Needed (for Plan phase)

1. **Language:** Rust (aligns with btop-like performance goals and competitor landscape)
2. **Primary data source:** Local JSONL files (zero auth) vs API-first
3. **Real-time vs polling:** JSONL files can be watched with inotify/kqueue; API polled every 60s
4. **Multi-provider strategy:** Plugin/adapter pattern for each provider?
5. **Session rate limit data:** Can we reverse-engineer the Claude Console session limit API? Or compute from local data?
6. **Distribution:** Cargo install, Homebrew, npm wrapper (like tokscale), or all three?

## 7. Architecture Sketch (Input for Plan)

```
┌─────────────────────────────────────────────┐
│                  TUI Layer                   │
│         (Ratatui + Crossterm)                │
│  ┌──────┬──────┬──────┬──────┐              │
│  │Dash  │Models│Sessions│Config│  ← Tab views│
│  └──────┴──────┴──────┴──────┘              │
├─────────────────────────────────────────────┤
│              Aggregation Layer               │
│    (merge + deduplicate + compute rates)     │
├─────────────────────────────────────────────┤
│              Data Source Adapters            │
│  ┌──────────┬──────────┬──────────┐         │
│  │ Local    │ Anthropic│ OpenAI   │         │
│  │ JSONL    │ Admin API│ Usage API│         │
│  └──────────┴──────────┴──────────┘         │
├─────────────────────────────────────────────┤
│              Config / Auth Store             │
│         (~/.config/aitop/config.toml)        │
└─────────────────────────────────────────────┘
```

## 8. Differentiation Summary

| Feature | tokscale | toktop | tu | ccusage | **aitop** |
|---------|----------|--------|----|---------|-----------|
| btop-style live TUI | Partial | No | Partial | No | **Yes** |
| API-level org usage | No | Yes | No | No | **Yes** |
| Local session parsing | Yes | No | Yes | Yes | **Yes** |
| Rate limit tracking | No | No | No | No | **Yes** |
| Multi-provider unified | Partial | Yes | No | No | **Yes** |
| Zero-auth mode | Yes | No | Yes | Yes | **Yes** |
| Burn rate / predictions | No | No | No | No | **Yes** |
| Single binary | Yes* | Yes | Yes | No | **Yes** |

*tokscale ships Rust native binaries via npm wrapper

## Sources

- [Anthropic Usage & Cost API docs](https://platform.claude.com/docs/en/build-with-claude/usage-cost-api)
- [Claude Code Monitoring docs](https://code.claude.com/docs/en/monitoring-usage)
- [Claude Code Cost Management](https://code.claude.com/docs/en/costs)
- [tokscale GitHub](https://github.com/junhoyeo/tokscale)
- [toktop on Terminal Trove](https://terminaltrove.com/toktop/)
- [ccusage GitHub](https://github.com/ryoppippi/ccusage)
- [tokenusage (tu) on lib.rs](https://lib.rs/crates/tokenusage)
- [Claude-Code-Usage-Monitor GitHub](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor)
- [Claude Code session file format](https://databunny.medium.com/inside-claude-code-the-session-file-format-and-how-to-inspect-it-b9998e66d56b)
