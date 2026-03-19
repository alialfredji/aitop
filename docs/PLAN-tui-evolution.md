# TUI Evolution Plan

## Vision

Evolve aitop's terminal interface from a dashboard viewer into a **terminal-native AI
operations center** — matching btop's polish while pioneering new paradigms for AI cost
monitoring that no web UI can replicate.

---

## Phase 1: Interaction Depth (v0.3)

### 1.1 Projects View — The Missing Tab

The current 4-view model (Dashboard, Sessions, Models, Trends) is missing the most
natural entry point: **projects**. Users think in projects, not sessions.

```
┌─ aitop ─── Dashboard │ Sessions │ Models │ Trends │ Projects ── ● LIVE ──┐
├──────────────────────────────────────────────────────────────────────────┤
│  PROJECT              │ TOTAL COST │ SESSIONS │ LAST ACTIVE │ 7d TREND  │
│  echopad              │   $342.10  │       23 │    12m ago  │ ▁▂▃▅▇▅▃  │
│  personal-site        │    $89.40  │       15 │     2h ago  │ ▃▃▂▁▁▁▁  │
│  dotfiles             │    $42.00  │        8 │     1d ago  │ ▁▁▂▃▂▁▁  │
│  aitop                │    $31.20  │       12 │     5m ago  │ ▁▁▁▂▅▇█  │
├──────────────────────────────────────────────────────────────────────────┤
│  PROJECT COST BREAKDOWN                                                  │
│  echopad       ████████████████████████████████████ 68%  $342           │
│  personal-site ██████████                           18%   $89           │
│  dotfiles      █████                                 8%   $42           │
│  aitop         ████                                  6%   $31           │
├──────────────────────────────────────────────────────────────────────────┤
│ aitop v0.3 │ 4 projects │ $504 all-time │ p:proj c:cost n:name │ ⟳ 2s │
└──────────────────────────────────────────────────────────────────────────┘
```

**Navigation:** `Enter` on a project drills into its sessions (filtered sessions view).
`Esc` returns to project list.

**Key binding resolution:** Move theme cycling from `p` to `F2` (or keep `p` as Projects
view and use `Shift+T` for theme). Projects is more important than quick theme cycling.

### 1.2 Breadcrumb Navigation

As we add drill-down paths (Projects → Sessions → Session Detail), show a breadcrumb:

```
 Projects > echopad > Session abc123
```

`Esc` pops one level. Clear mental model of "where am I."

### 1.3 Mouse Support

Optional mouse support (off by default, `mouse = true` in config):

- Click tab bar items to switch views
- Click table rows to select
- Scroll wheel for table/detail scrolling
- Hover effects on interactive elements (subtle underline)
- Right-click context menu (stretch goal)

Mouse users exist even in terminals. Supporting them costs little and expands reach.

### 1.4 Multi-Select in Tables

Mark multiple sessions with `x` or `Space`:

```
│   │ echopad     │ opus    │ $42.30 │
│ ✓ │ dotfiles    │ haiku   │  $1.20 │
│ ✓ │ personal    │ sonnet  │  $8.10 │
```

Actions on selection: compare (`c`), export (`e`), aggregate stats in status bar.

---

## Phase 2: Visual Richness (v0.4)

### 2.1 Animated Transitions

Smooth transitions between views using Ratatui's frame-by-frame rendering:

- **Tab switch:** Slide content left/right (3-5 frames, ~50ms)
- **Popup open:** Scale up from center (3 frames)
- **Popup close:** Fade to transparent (3 frames)
- **Delta banner:** Slide down from top, auto-dismiss slides up

Implementation: State machine with `Transition` enum, interpolate layout rects per frame.
Keep animations < 100ms total — perceptible but never blocking.

### 2.2 Braille-Resolution Charts

Replace block-character charts with braille-dot plotting for 2x resolution:

```
Current (block):  ▁▂▃▅▇█▇▅▃▂▁
Braille (2x res): ⠀⠁⠃⠇⠏⠟⠿⡿⣿⣿⡿⠿⠏⠇⠃⠁
```

Each braille character encodes a 2×4 dot matrix — effectively doubling both horizontal
and vertical resolution of sparklines and charts. The `drawille` algorithm maps
floating-point values to braille Unicode codepoints.

### 2.3 Dual-Axis Charts

Trends view daily spend chart with two Y-axes:

```
  $│                                          │tokens
 30│       ╭─╮                                │90K
 20│    ╭──╯ ╰──╮     ╭─╮                     │60K
 10│ ╭──╯       ╰─────╯ ╰──╮                  │30K
  0│─╯                      ╰────             │0
   └──────────────────────────────────────────
     Mon  Tue  Wed  Thu  Fri  Sat  Sun

     ── cost ($)    ·· tokens
```

Left axis: cost (solid line). Right axis: token count (dotted line).

### 2.4 Live Streaming Indicators

When file watcher detects active writing, show real-time token accumulation:

```
  ● LIVE  echopad — opus-4-6
  ┌──────────────────────────────────┐
  │  This minute: $0.42  (↑ $0.08)  │
  │  Tokens: 2.1K in / 8.4K out     │
  │  ▁▂▃▅▇█▇▅▃ (last 60s)          │
  └──────────────────────────────────┘
```

Pulse animation on the `●` indicator (alternate filled/empty on 500ms tick).

### 2.5 Responsive Panel Grid

Three layout tiers instead of two:

| Width | Layout |
|-------|--------|
| < 80 cols | **Compact:** Single column, stacked panels, abbreviated labels |
| 80-119 cols | **Standard:** Current 2-column layout |
| ≥ 120 cols | **Wide:** 3-column layout with side panels |

Height-aware: If terminal is < 30 rows, collapse chart panels to single-line sparklines.
If > 50 rows, expand activity feed and show more table rows.

---

## Phase 3: Terminal-Native Innovations (v0.5+)

### 3.1 Inline Images (Sixel/Kitty Protocol)

For terminals that support image protocols (iTerm2, WezTerm, Kitty, foot):

- Render high-resolution charts as actual images inline
- Contribution calendar with proper color gradients (not block chars)
- Model distribution as a real pie chart
- Detect terminal capabilities via `$TERM_PROGRAM` / XTVERSION

Fallback: standard block-character rendering for unsupported terminals.

### 3.2 Split Panes

`Ctrl+\` to split current view horizontally or vertically:

```
┌─ Dashboard ─────────────────┬─ Sessions ──────────────────┐
│  BURN RATE      SPEND TODAY │  # │ Project  │ Cost │ Upd  │
│  $2.34/hr       $18.72      │  1 │ echopad  │ $42  │ 12m  │
│                             │  2 │ dotfiles │  $1  │  2h  │
│  TOKEN FLOW (60m)           │  3 │ personal │  $8  │  3h  │
│  ▁▂▃▅▇█▇▅▃▂▁               │  4 │ aitop    │  $3  │  5m  │
└─────────────────────────────┴──────────────────────────────┘
```

View two tabs simultaneously. Active pane has bright border, inactive dim.

### 3.3 Session Replay

Replay a session's token activity as a time-lapse:

- `R` on session detail popup starts replay
- Fast-forward through message events
- Watch cost accumulate and token flow animate
- Speed controls: 1x, 2x, 5x, 10x
- Useful for understanding "where did the money go?"

### 3.4 Terminal Bell / Sound Alerts

- Bell (`\a`) when budget threshold crossed
- Configurable alert sounds for anomalies
- Config: `alerts.bell = true`, `alerts.budget_threshold = 90`

### 3.5 Tmux Integration

- Set tmux status bar segment with current burn rate
- `aitop --tmux-status` outputs single-line for `#(aitop --tmux-status)`
- Format: `AI: $2.34/hr ● LIVE` or `AI: idle`

---

## Phase 4: Beyond Monitoring (v1.0 vision)

### 4.1 Budget Controls

Move from monitoring to **governance**:

- Hard spend limits with session-kill capability
- Per-project budgets with alerts
- Daily/weekly/monthly budget modes
- "Pause AI" button that writes a sentinel file Claude Code respects

(Requires upstream integration — Claude Code would need to check budget files.)

### 4.2 Team Dashboard

When Admin API is connected, show team-wide stats:

```
  TEAM OVERVIEW (org: acme-corp)
  ┌────────────┬──────────┬──────────┬──────────┐
  │ Member     │ Today    │ This Wk  │ Model    │
  ├────────────┼──────────┼──────────┼──────────┤
  │ alice      │   $12.30 │   $89.40 │ opus     │
  │ bob        │    $3.20 │   $42.10 │ sonnet   │
  │ carol      │   $18.90 │  $142.30 │ opus     │
  └────────────┴──────────┴──────────┴──────────┘
```

### 4.3 AI-Powered Insights

Use Claude API to analyze usage patterns and generate natural-language insights:

```
  💡 INSIGHTS
  "Your opus usage increased 3x this week, primarily on echopad.
   Consider switching routine tasks to sonnet — estimated savings: $45/week.
   Your peak usage is 10am-12pm on Tuesdays."
```

Opt-in, runs on-demand (`i` key), uses a single haiku call for cost efficiency.

### 4.4 Plugin System

Allow community extensions:

```toml
[plugins]
cost-alerts = { path = "~/.config/aitop/plugins/alerts.wasm" }
custom-view = { git = "https://github.com/user/aitop-plugin" }
```

WASM-based plugins for safety and portability. Plugin API:
- `on_refresh(data: &AppData)` — react to data changes
- `render(frame: &mut Frame, area: Rect)` — custom view rendering
- `handle_key(key: KeyEvent) -> Action` — custom key bindings

---

## Design Principles

1. **Terminal-first.** Never build a web UI. The terminal is the native habitat of
   developers who use AI coding agents. Optimize for that context.

2. **Zero-config useful.** Every feature works with defaults. Config is for power users.
   First run should show useful data within 200ms.

3. **Information density over whitespace.** Every pixel of terminal space should convey
   information. Follow btop's example: dense, colorful, alive.

4. **Keyboard-first, mouse-optional.** All functionality accessible via keyboard.
   Mouse support is a convenience layer, never required.

5. **Non-destructive.** aitop only reads data. It never modifies session files,
   sends network requests (unless Admin API is configured), or affects Claude Code's
   behavior.

6. **Fast always.** 60fps rendering, <200ms startup, <16ms per frame. If a feature
   can't hit these targets, it needs optimization before shipping.

---

## View Evolution Summary

| Version | Views | Key Addition |
|---------|-------|--------------|
| v0.1 | Dashboard, Sessions, Models, Trends | Foundation |
| v0.2 | Same 4 + overlays (Help, Filter, Detail) | Interaction depth |
| v0.3 | + Projects (5th tab) | Project-centric navigation |
| v0.4 | + Split panes | Multi-view simultaneously |
| v0.5 | + Inline images, replay | Visual richness |
| v1.0 | + Team, Insights, Plugins | Platform |
