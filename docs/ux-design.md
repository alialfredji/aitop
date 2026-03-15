# UX Design

## Philosophy

Steal btop's DNA:
- **Highlighted shortcut letters** — first letter of each tab is accent-colored + underlined
- **Information density** — no wasted space, every cell shows something useful
- **Responsive** — adapts to terminal size (compact < 100 cols, wide ≥ 100)
- **Live** — 2s refresh tick by default, updates in-place
- **Keyboard-first** — vim keys, global nav, view-specific actions

## Color Palette (Default "ember")

```
Primary:     #FF6B35 (warm orange — accent, shortcuts, burn rate)
Secondary:   #4ECDC4 (teal — input tokens, cache)
Tertiary:    #FFE66D (gold — output tokens, cost numbers)
Muted:       #555555 (borders, dim labels)
Danger:      #FF1744 (over budget, high burn)
Success:     #00E676 (under budget, low burn)
Background:  terminal default (transparent-friendly)
```

Themes: ember, nord, dracula, gruvbox, catppuccin, mono

## Layout (wide ≥ 120 cols)

```
┌─ aitop ─────────────── Dashboard │ Sessions │ Models │ Trends ── ● LIVE ──┐
├──────────────────────────────┬────────────────────────────────────────────┤
│  BURN RATE      SPEND TODAY  │  TOKEN FLOW (last 60 min)                  │
│  $2.34/hr ▲     $18.72      │  ▁▂▃▅▇█▇▅▃▂▁▁▂▄▆█▇▅▃▂▁▁▁▂▃▅▇█▇▅▃▁      │
│                               │                                           │
│  This Week      Budget       │  CACHE HIT RATIO                          │
│  $142 / $200  ████████░░ 71% │  ████████████████░░░░ 82%                 │
├──────────────────────────────┼────────────────────────────────────────────┤
│  MODEL BREAKDOWN              │  ACTIVE SESSIONS                          │
│  opus-4-6  ████████████ $312  │  echopad    opus   $4.20  12m ago        │
│  haiku-4-5 ██           $10  │  dotfiles   haiku  $0.31   2h ago        │
│  sonnet    █             $4  │  personal   opus   $8.10   3h ago        │
├──────────────────────────────┴────────────────────────────────────────────┤
│  RECENT ACTIVITY                                                          │
│  14:23  echopad  opus-4-6   324in/1.2Kout/18Kc  $0.08                   │
│  14:21  echopad  opus-4-6   289in/891out/18Kc    $0.06                   │
├───────────────────────────────────────────────────────────────────────────┤
│ aitop v0.1 │ 85 sessions │ $1,905 │ d:dash s:sess m:mod t:trend │ ⟳ 2s  │
└───────────────────────────────────────────────────────────────────────────┘
```

## Keyboard Map

### Global (always active)
| Key | Action |
|-----|--------|
| `d` / `1` | **D**ashboard |
| `s` / `2` | **S**essions |
| `m` / `3` | **M**odels |
| `t` / `4` | **T**rends |
| `q` / `Ctrl-C` | Quit |
| `?` / `F1` | Help overlay |
| `r` | Force refresh |
| `p` | Cycle theme |
| `/` | Filter/search |
| `Esc` | Close overlay |

### Sessions view
| Key | Action |
|-----|--------|
| `j`/`k` `↑`/`↓` | Navigate rows |
| `Enter` | Drill into session detail |
| `c` | Sort by cost |
| `n` | Sort by tokens |
| `p` | Sort by project |
| `u` | Sort by updated |

### Trends view
| Key | Action |
|-----|--------|
| `w` | Last week |
| `o` | Last month |
| `a` | All time |
| `←`/`→` | Cycle range |

## Planned UX Features

### Session Detail Popup
Enter on a session → centered overlay showing message-by-message cost bars, token distribution, cache efficiency.

### Inline Sparklines
Tiny `▁▂▃▅▇` sparklines in session table cells showing 7-day cost trend per session.

### Cost Color Gradient
Dollar amounts color-coded: green ($0-1) → yellow ($1-5) → orange ($5-10) → red ($10+).

### "Since Last Check" Banner
On startup, show what changed since you last looked: spend delta, new sessions, model mix changes. Auto-dismisses after 10s.

### Time-of-Day Heatmap
7×24 grid showing when you burn tokens most, using block chars: ░▒▓█

### Budget Gauge
Color-threshold progress bar: green < 60%, yellow 60-85%, red > 85%.
