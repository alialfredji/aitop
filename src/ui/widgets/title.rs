use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::ui::theme::Theme;

/// Panel title with an underlined shortcut key hint.
/// The `shortcut` character is rendered underlined + accent color,
/// the rest of the `label` in normal text.
///
/// Example: `shortcut_title('M', "odels ", theme)` → **M**odels
pub fn shortcut_title<'a>(shortcut: char, rest: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            shortcut.to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ),
        Span::styled(rest, Style::default().fg(theme.text)),
    ])
}

/// Plain panel title — bold accent, no underline. Use for panels
/// that are not directly navigable via a shortcut key.
///
/// Example: `panel_title("Burn Rate", theme)` → **Burn Rate**
pub fn panel_title<'a>(label: &'a str, theme: &Theme) -> Line<'a> {
    Line::from(vec![Span::styled(
        label,
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::get_theme;

    #[test]
    fn test_shortcut_title_has_underline() {
        let theme = get_theme("ember");
        let line = shortcut_title('M', "odels ", &theme);
        assert_eq!(line.spans.len(), 2);
        assert!(line.spans[0]
            .style
            .add_modifier
            .contains(Modifier::UNDERLINED));
        assert_eq!(line.spans[0].content, "M");
    }

    #[test]
    fn test_panel_title_no_underline() {
        let theme = get_theme("ember");
        let line = panel_title("Burn Rate ", &theme);
        assert_eq!(line.spans.len(), 1);
        assert!(!line.spans[0]
            .style
            .add_modifier
            .contains(Modifier::UNDERLINED));
    }
}
