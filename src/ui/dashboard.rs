use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Sparkline, Table};
use ratatui::Frame;

use super::layout::{dashboard_layout, is_wide};
use super::theme::Theme;
use crate::app::AppState;

pub fn render_dashboard(f: &mut Frame, state: &AppState, theme: &Theme) {
    let wide = is_wide(f.area());
    // Use the content area (after tab bar)
    let areas = dashboard_layout(state.content_area, wide);

    render_metrics(f, state, theme, areas.metrics);
    render_token_flow(f, state, theme, areas.token_flow);
    render_model_breakdown(f, state, theme, areas.model_breakdown);
    if wide && areas.sessions.width > 0 {
        render_active_sessions(f, state, theme, areas.sessions);
    }
    render_activity_feed(f, state, theme, areas.activity);
}

fn render_metrics(f: &mut Frame, state: &AppState, theme: &Theme, area: ratatui::layout::Rect) {
    let stats = &state.dashboard;

    let burn_arrow = if stats.burn_rate_per_hour > 0.0 { "▲" } else { "·" };
    let burn_color = if stats.burn_rate_per_hour > 10.0 {
        theme.danger
    } else if stats.burn_rate_per_hour > 5.0 {
        theme.tertiary
    } else {
        theme.success
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.muted))
        .title(Line::from(vec![
            Span::styled(" B", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("urn Rate ", Style::default().fg(theme.text)),
        ]));

    // Split metrics area into left (burn rate) and right (spend)
    let inner = block.inner(area);
    f.render_widget(block, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Left: burn rate big number
    let burn_text = vec![
        Line::from(vec![
            Span::styled(
                format!("${:.2}/hr {}", stats.burn_rate_per_hour, burn_arrow),
                Style::default().fg(burn_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Today ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("${:.2}", stats.spend_today),
                Style::default().fg(theme.tertiary).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(burn_text), cols[0]);

    // Right: week spend + budget gauge
    let mut right_lines = vec![
        Line::from(vec![
            Span::styled("This Week ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("${:.2}", stats.spend_this_week),
                Style::default().fg(theme.tertiary).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if let Some(budget) = state.config.weekly_budget {
        let pct = (stats.spend_this_week / budget).min(1.0);
        right_lines.push(Line::from(vec![
            Span::styled(
                format!("${:.0} / ${:.0}", stats.spend_this_week, budget),
                Style::default().fg(if pct > 0.9 { theme.danger } else { theme.text_dim }),
            ),
        ]));
    } else {
        right_lines.push(Line::from(vec![
            Span::styled("All-time ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("${:.2}", stats.spend_all_time),
                Style::default().fg(theme.text)),
        ]));
    }

    f.render_widget(Paragraph::new(right_lines), cols[1]);
}

fn render_token_flow(f: &mut Frame, state: &AppState, theme: &Theme, area: ratatui::layout::Rect) {
    let data: Vec<u64> = if state.token_flow.is_empty() {
        vec![0; 60]
    } else {
        state.token_flow.iter().map(|p| p.total_tokens as u64).collect()
    };

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.muted))
                .title(Line::from(vec![
                    Span::styled(" T", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
                    Span::styled("oken Flow ", Style::default().fg(theme.text)),
                    Span::styled("(last hour) ", Style::default().fg(theme.text_dim)),
                ])),
        )
        .data(&data)
        .style(Style::default().fg(theme.secondary));

    f.render_widget(sparkline, area);
}

fn render_model_breakdown(f: &mut Frame, state: &AppState, theme: &Theme, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.muted))
        .title(Line::from(vec![
            Span::styled(" M", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("odels ", Style::default().fg(theme.text)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.models.is_empty() {
        f.render_widget(
            Paragraph::new("  No data yet").style(Style::default().fg(theme.text_dim)),
            inner,
        );
        return;
    }

    let max_cost = state.models.iter().map(|m| m.cost).fold(0.0f64, f64::max);
    let bar_width = (inner.width as usize).saturating_sub(30);

    let mut lines = Vec::new();
    for model in &state.models {
        let short_name = shorten_model(&model.model);
        let bar_len = if max_cost > 0.0 {
            ((model.cost / max_cost) * bar_width as f64) as usize
        } else {
            0
        };
        let bar: String = "█".repeat(bar_len);
        let empty: String = "░".repeat(bar_width.saturating_sub(bar_len));

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<14}", short_name),
                Style::default().fg(theme.text),
            ),
            Span::styled(bar, Style::default().fg(theme.bar_filled)),
            Span::styled(empty, Style::default().fg(theme.bar_empty)),
            Span::styled(
                format!(" ${:.2}", model.cost),
                Style::default().fg(theme.tertiary).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_active_sessions(f: &mut Frame, state: &AppState, theme: &Theme, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.muted))
        .title(Line::from(vec![
            Span::styled(" S", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("essions ", Style::default().fg(theme.text)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows: Vec<Row> = state
        .sessions
        .iter()
        .take(inner.height as usize)
        .enumerate()
        .map(|(i, s)| {
            let style = if i % 2 == 0 {
                Style::default().fg(theme.text)
            } else {
                Style::default().fg(theme.text_dim)
            };
            Row::new(vec![
                format!("{:<12}", truncate(&s.project, 12)),
                shorten_model(&s.model),
                format!("${:.2}", s.total_cost),
                format_relative_time(&s.updated_at),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(13),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Project", "Model", "Cost", "When"])
            .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    );

    f.render_widget(table, inner);
}

fn render_activity_feed(f: &mut Frame, state: &AppState, theme: &Theme, area: ratatui::layout::Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.muted))
        .title(Line::from(vec![
            Span::styled(" R", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("ecent Activity ", Style::default().fg(theme.text)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows: Vec<Row> = state
        .activity
        .iter()
        .take(inner.height as usize)
        .map(|a| {
            let time = a.timestamp.get(11..16).unwrap_or("??:??");
            Row::new(vec![
                time.to_string(),
                format!("{:<12}", truncate(&a.project, 12)),
                shorten_model(&a.model),
                format!("{}in/{}out", format_tokens(a.input_tokens), format_tokens(a.output_tokens)),
                format!("{}c", format_tokens(a.cache_read)),
                format!("${:.3}", a.cost_usd),
            ])
            .style(Style::default().fg(theme.text_dim))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(13),
            Constraint::Length(10),
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["Time", "Project", "Model", "Tokens", "Cache", "Cost"])
            .style(Style::default().fg(theme.accent)),
    );

    f.render_widget(table, inner);
}

// --- Helpers ---

fn shorten_model(model: &str) -> String {
    model
        .replace("claude-", "")
        .replace("-20241022", "")
        .replace("-20250514", "")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max - 1])
    } else {
        s.to_string()
    }
}

fn format_tokens(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_relative_time(iso: &str) -> String {
    use chrono::{DateTime, Utc};
    let Ok(dt) = iso.parse::<DateTime<Utc>>() else {
        return iso.to_string();
    };
    let now = Utc::now();
    let diff = now - dt;

    if diff.num_minutes() < 1 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else {
        format!("{}d ago", diff.num_days())
    }
}
