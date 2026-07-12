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

const BUCKETS: u16 = 5;
const BUCKET_GAP: u16 = 1;
/// Blank columns between the box walls and the outermost buckets;
/// matches the inter-bucket gap.
const BUCKET_PAD: u16 = BUCKET_GAP;

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
    let widths = bucket_widths(cells - 2 * BUCKET_PAD, BUCKETS, BUCKET_GAP);
    let filled = filled_buckets(snapshot.percent);
    let color = level_color(snapshot.percent);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(format!("┌{}┐", "─".repeat(cells as usize))));
    for row in 0..FILL_ROWS {
        let nub = if row == FILL_ROWS / 2 { "▌" } else { "" };
        let mut spans = vec![Span::raw("│"), Span::raw(" ".repeat(BUCKET_PAD as usize))];
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" ".repeat(BUCKET_GAP as usize)));
            }
            if (i as u16) < filled {
                spans.push(Span::styled("█".repeat(*w as usize), Style::new().fg(color)));
            } else {
                spans.push(Span::styled(
                    "░".repeat(*w as usize),
                    Style::new().fg(EMPTY_FILL),
                ));
            }
        }
        spans.push(Span::raw(" ".repeat(BUCKET_PAD as usize)));
        spans.push(Span::raw("│"));
        spans.push(Span::raw(nub));
        lines.push(Line::from(spans));
    }
    lines.push(Line::from(format!("└{}┘", "─".repeat(cells as usize))));
    lines.push(Line::default());
    // Center the status text under the box (nub excluded).
    let mut status = status_line(snapshot, color);
    let pad = body_w.saturating_sub(status.width() as u16) / 2;
    status.spans.insert(0, Span::raw(" ".repeat(pad as usize)));
    lines.push(status);

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
    match filled_buckets(percent) {
        0 | 1 => Color::Red,
        2 => Color::Indexed(208), // orange; no named ANSI-16 equivalent
        3 => Color::Yellow,
        4 => Color::LightGreen,
        _ => Color::Green,
    }
}

/// Buckets lit for a charge percentage: one per started 20% band.
fn filled_buckets(percent: u8) -> u16 {
    u16::from(percent).div_ceil(20)
}

/// Per-bucket column widths; the first `usable % buckets` get the remainder.
fn bucket_widths(cells: u16, buckets: u16, gap: u16) -> Vec<u16> {
    let usable = cells.saturating_sub(gap * (buckets - 1));
    let base = usable / buckets;
    let rem = usable % buckets;
    (0..buckets).map(|i| base + u16::from(i < rem)).collect()
}

fn format_hm(minutes: u32) -> String {
    format!("{}:{:02}", minutes / 60, minutes % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filled_buckets_band_edges() {
        assert_eq!(filled_buckets(0), 0);
        assert_eq!(filled_buckets(1), 1);
        assert_eq!(filled_buckets(20), 1);
        assert_eq!(filled_buckets(21), 2);
        assert_eq!(filled_buckets(80), 4);
        assert_eq!(filled_buckets(81), 5);
        assert_eq!(filled_buckets(100), 5);
    }

    #[test]
    fn bucket_widths_fill_the_row_exactly() {
        // 10 and 42 are the min/max bucket areas after borders and padding.
        for cells in [10, 29, 42] {
            let widths = bucket_widths(cells, BUCKETS, BUCKET_GAP);
            let total: u16 = widths.iter().sum::<u16>() + BUCKET_GAP * (BUCKETS - 1);
            assert_eq!(total, cells, "cells={cells}");
            assert!(widths.windows(2).all(|w| w[0] >= w[1] && w[0] - w[1] <= 1));
        }
    }

    #[test]
    fn format_hm_pads_minutes() {
        assert_eq!(format_hm(0), "0:00");
        assert_eq!(format_hm(72), "1:12");
        assert_eq!(format_hm(605), "10:05");
    }
}
