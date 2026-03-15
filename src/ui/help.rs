use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use super::theme::Theme;

pub fn render_help(f: &mut Frame, theme: &Theme) {
    let area = centered_rect(60, 70, f.area());

    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![
            Span::styled("aitop", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" — btop for AI", Style::default().fg(theme.text_dim)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
        shortcut_line("d", "Dashboard", theme),
        shortcut_line("s", "Sessions", theme),
        shortcut_line("m", "Models", theme),
        shortcut_line("t", "Trends", theme),
        shortcut_line("1-4", "Quick switch view", theme),
        shortcut_line("Tab", "Cycle panels", theme),
        Line::from(""),
        Line::from(Span::styled("Actions", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
        shortcut_line("/", "Search / filter", theme),
        shortcut_line("r", "Force refresh", theme),
        shortcut_line("+/-", "Adjust refresh rate", theme),
        shortcut_line("Esc", "Close overlay / clear", theme),
        shortcut_line("q", "Quit", theme),
        Line::from(""),
        Line::from(Span::styled("In Tables", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
        shortcut_line("↑/↓ j/k", "Navigate rows", theme),
        shortcut_line("Enter", "Drill into detail", theme),
        shortcut_line("c", "Sort by cost", theme),
        shortcut_line("n", "Sort by tokens", theme),
        shortcut_line("p", "Sort by project", theme),
        Line::from(""),
        Line::from(Span::styled("In Trends", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
        shortcut_line("w", "Last week", theme),
        shortcut_line("m", "Last month", theme),
        shortcut_line("a", "All time", theme),
    ];

    let para = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .title(" Help (? to close) ")
            .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    );

    f.render_widget(para, area);
}

fn shortcut_line<'a>(key: &'a str, desc: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:>10}", key),
            Style::default().fg(theme.tertiary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {}", desc), Style::default().fg(theme.text)),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}
