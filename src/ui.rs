use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::App;
use crate::battery::{ChargeState, Snapshot};

// PanEx family theme (~/.claude/themes/panex-tui-style.md)
const HINT: Color = Color::DarkGray;
const STATUS_MSG: Color = Color::Yellow;
const EMPTY_FILL: Color = Color::DarkGray;

const FILL_ROWS: u16 = 3;
const MAX_BODY_WIDTH: u16 = 46;
const MIN_BODY_WIDTH: u16 = 14;

/// Inline-viewport height: battery box + blank + status line + hints bar.
pub const VIEWPORT_HEIGHT: u16 = FILL_ROWS + 5;

pub fn draw(frame: &mut Frame, app: &App, show_bar: bool) {
    let [content, bar] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    draw_battery(frame, content, &app.snapshot);
    // The final pre-exit frame omits the bar so the key hints don't end up
    // in the scrollback.
    if show_bar {
        draw_bar(frame, bar, app);
    }
}

fn draw_battery(frame: &mut Frame, area: Rect, snapshot: &Snapshot) {
    let body_w = area
        .width
        .saturating_sub(2)
        .clamp(MIN_BODY_WIDTH, MAX_BODY_WIDTH);
    // Center the art block (box plus the 1-column nub) horizontally.
    let inset = area.width.saturating_sub(body_w + 1) / 2;
    let area = Rect {
        x: area.x + inset,
        width: area.width.saturating_sub(inset),
        ..area
    };
    let cells = body_w - 2;
    let filled = fill_cells(snapshot.percent, cells);
    let color = level_color(snapshot.percent);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(format!("┌{}┐", "─".repeat(cells as usize))));
    for row in 0..FILL_ROWS {
        let nub = if row == FILL_ROWS / 2 { "▌" } else { "" };
        lines.push(Line::from(vec![
            Span::raw("│"),
            Span::styled("█".repeat(filled as usize), Style::new().fg(color)),
            Span::styled(
                "░".repeat((cells - filled) as usize),
                Style::new().fg(EMPTY_FILL),
            ),
            Span::raw("│"),
            Span::raw(nub),
        ]));
    }
    lines.push(Line::from(format!("└{}┘", "─".repeat(cells as usize))));
    lines.push(Line::default());
    lines.push(status_line(snapshot, color));

    frame.render_widget(Paragraph::new(lines), area);
}

fn status_line(snapshot: &Snapshot, color: Color) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            format!("{}%", snapshot.percent),
            Style::new().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
    ];
    let time = snapshot.minutes.map(format_hm);
    match snapshot.state {
        ChargeState::Charging => {
            spans.push(Span::styled("⚡ ", Style::new().fg(STATUS_MSG)));
            spans.push(Span::styled(
                match time {
                    Some(t) => format!("{t} to full"),
                    None => "charging".into(),
                },
                Style::new().fg(HINT),
            ));
        }
        ChargeState::Discharging => {
            spans.push(Span::styled(
                match time {
                    Some(t) => format!("{t} left"),
                    None => "on battery".into(),
                },
                Style::new().fg(HINT),
            ));
        }
        ChargeState::Full => {
            spans.push(Span::styled("✓ ", Style::new().fg(Color::Green)));
            spans.push(Span::styled("full", Style::new().fg(HINT)));
        }
        ChargeState::Empty => {
            spans.push(Span::styled("empty", Style::new().fg(Color::Red)));
        }
        ChargeState::Idle => {
            spans.push(Span::styled(
                "plugged in · not charging",
                Style::new().fg(HINT),
            ));
        }
    }
    Line::from(spans)
}

fn draw_bar(frame: &mut Frame, area: Rect, app: &App) {
    let line = match &app.status {
        Some(msg) => Line::styled(format!(" {msg}"), Style::new().fg(STATUS_MSG)),
        None => Line::styled(" q:quit │ r:refresh", Style::new().fg(HINT)),
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn level_color(percent: u8) -> Color {
    match percent {
        0..=20 => Color::Red,
        21..=50 => Color::Yellow,
        _ => Color::Green,
    }
}

/// Filled cells for a charge percentage, rounded to nearest.
fn fill_cells(percent: u8, cells: u16) -> u16 {
    ((u32::from(percent) * u32::from(cells) + 50) / 100) as u16
}

fn format_hm(minutes: u32) -> String {
    format!("{}:{:02}", minutes / 60, minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_cells_bounds_and_rounding() {
        assert_eq!(fill_cells(0, 30), 0);
        assert_eq!(fill_cells(100, 30), 30);
        assert_eq!(fill_cells(50, 30), 15);
        assert_eq!(fill_cells(1, 30), 0); // 0.3 rounds down
        assert_eq!(fill_cells(2, 30), 1); // 0.6 rounds up
        assert_eq!(fill_cells(99, 30), 30); // 29.7 rounds up
    }

    #[test]
    fn format_hm_pads_minutes() {
        assert_eq!(format_hm(0), "0:00");
        assert_eq!(format_hm(72), "1:12");
        assert_eq!(format_hm(605), "10:05");
    }
}
