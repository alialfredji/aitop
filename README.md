# aitop

**btop for AI** — a terminal dashboard for monitoring AI token usage, costs, and sessions.

Like `btop` monitors your system resources, `aitop` monitors your AI spend. Built for developers who live in the terminal and want to keep an eye on their Claude Code (and eventually other AI) costs without leaving it.

## Features

- **Live TUI dashboard** with btop-style keyboard shortcuts and highlighted shortcut letters
- **4 views**: Dashboard, Sessions, Models, Trends — switch with `d`/`s`/`m`/`t`
- **Zero auth required** — reads Claude Code's local JSONL session files directly
- **SQLite-backed** — indexes once, instant startup after first run
- **Burn rate tracking** — see your $/hr, daily spend, weekly totals at a glance
- **Model breakdown** — cost, tokens, and cache hit ratio per model
- **Session explorer** — sortable table of all sessions with drill-down
- **Spend trends** — daily cost chart with projections and averages
- **6 color themes** — ember (default), nord, dracula, gruvbox, catppuccin, mono
- **3MB single binary**, zero runtime dependencies, zero network calls

## Install

```bash
cargo install --path .
```

Or build from source:

```bash
git clone https://github.com/saurabhmishra/aitop.git
cd aitop
cargo build --release
./target/release/aitop
```

## Usage

```bash
# Launch TUI dashboard
aitop

# Quick table output (non-interactive)
aitop --light

# Choose a theme
aitop --theme dracula

# Custom refresh rate (seconds)
aitop --refresh 5
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `d` | **D**ashboard view |
| `s` | **S**essions view |
| `m` | **M**odels view |
| `t` | **T**rends view |
| `1`-`4` | Quick switch views |
| `j`/`k` or `↑`/`↓` | Navigate tables |
| `c`/`n`/`p` | Sort by cost/tokens/project |
| `r` | Force refresh |
| `?` | Help overlay |
| `q` | Quit |

## How It Works

`aitop` reads Claude Code session files from `~/.claude/projects/` and indexes them into a local SQLite database at `~/.local/share/aitop/sessions.db`. Token costs are computed using current Anthropic pricing. The database is incrementally updated — only new data in JSONL files is parsed on subsequent runs.

## Configuration

Config file at `~/.config/aitop/config.toml`:

```toml
refresh = 2          # Refresh interval in seconds
theme = "ember"      # Color theme
# weekly_budget = 200.0  # Optional budget gauge
```

## Tech Stack

Rust, Ratatui, Crossterm, SQLite (rusqlite), notify (fs watcher), Tokio

## License

MIT
