use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Whether the terminal is wide enough for side-by-side panels.
pub fn is_wide(area: Rect) -> bool {
    area.width >= 100
}

/// Split content area horizontally 50/50 for split-pane mode.
pub fn split_content(content: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content);
    (chunks[0], chunks[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_layout_proportions() {
        let area = Rect::new(0, 0, 120, 40);
        let (tab, content, status) = main_layout(area);
        assert_eq!(tab.height, 3, "tab bar should be 3 rows");
        assert_eq!(status.height, 1, "status bar should be 1 row");
        assert_eq!(content.height, 36, "content should fill remaining space");
        assert_eq!(tab.width, 120);
        assert_eq!(content.width, 120);
        assert_eq!(status.width, 120);
    }

    #[test]
    fn test_split_content_proportions() {
        let content = Rect::new(0, 3, 120, 36);
        let (left, right) = split_content(content);

        assert_eq!(left.width, 60, "left pane should be 50%");
        assert_eq!(right.width, 60, "right pane should be 50%");
        assert_eq!(left.height, content.height);
        assert_eq!(right.height, content.height);
        assert_eq!(left.x, content.x, "left pane starts at content left");
        assert_eq!(right.x, 60, "right pane starts at midpoint");
    }

    #[test]
    fn test_split_content_odd_width() {
        let content = Rect::new(0, 3, 101, 36);
        let (left, right) = split_content(content);

        // Ratatui's 50/50 split rounds; total should cover full width
        assert!(
            left.width + right.width >= 100 && left.width + right.width <= 101,
            "left + right should approximately equal content width: {} + {} vs {}",
            left.width, right.width, content.width
        );
    }
}

/// Split the main area into: tab bar (3 rows) + content area + status bar (1 row).
pub fn main_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

/// Dashboard: 2x2 grid + bottom activity feed.
pub fn dashboard_layout(area: Rect, wide: bool) -> DashboardAreas {
    if wide {
        // Wide: top row (2 cols) + mid row (2 cols) + bottom activity
        let vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),  // top metrics row
                Constraint::Min(8),    // mid row
                Constraint::Length(8), // activity feed
            ])
            .split(area);

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(vert[0]);

        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(vert[1]);

        DashboardAreas {
            metrics: top[0],
            token_flow: top[1],
            model_breakdown: mid[0],
            sessions: mid[1],
            activity: vert[2],
        }
    } else {
        // Compact: stacked vertically, tuned for 80x24 (20 content rows available)
        // Reduce fixed allocations so model_breakdown (Min) always gets space
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),  // metrics
                Constraint::Length(4),  // token flow (compact sparkline)
                Constraint::Min(3),    // model breakdown (gets remaining)
                Constraint::Length(6), // activity feed
            ])
            .split(area);

        DashboardAreas {
            metrics: chunks[0],
            token_flow: chunks[1],
            model_breakdown: chunks[2],
            sessions: Rect::default(), // hidden in compact
            activity: chunks[3],
        }
    }
}

pub struct DashboardAreas {
    pub metrics: Rect,
    pub token_flow: Rect,
    pub model_breakdown: Rect,
    pub sessions: Rect,
    pub activity: Rect,
}
