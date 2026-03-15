# Research

## Problem

Developers using AI coding agents have no real-time, unified terminal dashboard for monitoring token spend, session activity, and burn rate.

## Competitive Landscape

| Tool | Stack | Approach | Strengths | Gaps |
|------|-------|----------|-----------|------|
| [tokscale](https://github.com/junhoyeo/tokscale) | Rust+Node, Ratatui | Reads local JSONL from 16+ agents | Best coverage, 4 TUI views, 2D/3D contribution graphs, leaderboard | No API-level org usage, no rate limits, npm wrapper bloat |
| [toktop](https://github.com/htin1/toktop) | Rust | Calls OpenAI/Anthropic usage APIs | True API-level usage, multi-provider | Requires Admin API key, limited UI |
| [tokenusage (tu)](https://lib.rs/crates/tokenusage) | Rust, Ratatui+iced | Reads local JSONL | Blazing fast (0.08s), TUI+GUI+image cards | Local files only |
| [ccusage](https://github.com/ryoppippi/ccusage) | TypeScript | Reads local JSONL | 5-hour billing windows, MCP integration | Table output only, not interactive |
| [Claude-Code-Usage-Monitor](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor) | Python | Reads local JSONL | Burn rate predictions | Single-tool, no multi-provider |

**No existing tool combines:** real-time btop-style dashboard + API-level org usage + local session data + rate limits + multi-provider + zero-auth mode.

## Data Sources

### Local JSONL (primary — zero auth)
- Location: `~/.claude/projects/<url-encoded-project-path>/<session-uuid>.jsonl`
- Format: typed message records linked by `parentUuid`
- Every assistant turn has `usage` block: `input_tokens`, `output_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`
- Known issue: output tokens may be undercounted (streaming chunks only)

### Anthropic Admin API (org users only)
- Usage: `GET /v1/organizations/usage_report/messages` — buckets: 1m/1h/1d
- Cost: `GET /v1/organizations/cost_report` — daily granularity, USD cents
- Requires `sk-ant-admin...` key (org admin only)
- Polling: 1/min sustained, data freshness ~5 min

### Claude Code OpenTelemetry (opt-in)
- Metrics: `claude_code.token.usage`, `claude_code.cost.usage`, `claude_code.session.count`
- `CLAUDE_CODE_ENABLE_TELEMETRY=1` + OTLP/Prometheus/console exporters

### Claude Console Web UI
- Session/weekly limit data visible in web UI but **no public API**

## Sources

- [Anthropic Usage & Cost API](https://platform.claude.com/docs/en/build-with-claude/usage-cost-api)
- [Claude Code Monitoring](https://code.claude.com/docs/en/monitoring-usage)
- [Claude Code Costs](https://code.claude.com/docs/en/costs)
- [Claude Code session file format](https://databunny.medium.com/inside-claude-code-the-session-file-format-and-how-to-inspect-it-b9998e66d56b)
