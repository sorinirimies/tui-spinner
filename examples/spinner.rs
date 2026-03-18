//! # Spinner Widget Example
//!
//! Demonstrates all spinner widgets and style variants side-by-side:
//!
//! - **Col 1** — [`RectSpinner`] `Square`, filled centre, clockwise
//! - **Col 2** — [`RectSpinner`] `Square`, empty centre, counter-clockwise
//! - **Col 3** — [`RectSpinner`] `Narrow` — 1-char-wide Zed sidebar style
//! - **Col 4** — [`CircleSpinner`] — various radii, CW and CCW
//! - **Col 5** — [`LinearSpinner`] vertical and horizontal
//!
//! **Controls:**
//! - `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example spinner`

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Padding, Paragraph},
    DefaultTerminal, Frame,
};
use std::time::{Duration, Instant};
use tui_spinner::{
    Centre, CircleSpinner, Direction, LinearSpinner, LinearStyle, RectShape, RectSpinner,
    RectStyle, Spin,
};

// ── App state ─────────────────────────────────────────────────────────────────

struct App {
    tick: u64,
    last_tick: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tick: 0,
            last_tick: Instant::now(),
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut app = App::default();
    let terminal = ratatui::init();
    let result = run(terminal, &mut app);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        let now = Instant::now();
        let delta = now.duration_since(app.last_tick);
        app.last_tick = now;

        let steps = (delta.as_millis() / 80).max(1) as u64;
        app.tick = app.tick.wrapping_add(steps);

        terminal.draw(|frame| render(frame, app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ── Top-level layout ──────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let [header, content, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    render_header(frame, header);
    render_content(frame, content, app.tick);
    render_footer(frame, footer);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title(" tui-spinner Demo ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(1));

    let text = Paragraph::new(
        "Square · Filled · CW  ·  Square · Empty · CCW  ·  Narrow  ·  Circle  ·  LinearSpinner",
    )
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Gray));

    frame.render_widget(text.block(block), area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));

    let text = Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" / "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  Quit"),
    ]);

    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

fn render_content(frame: &mut Frame, area: Rect, tick: u64) {
    let [col_filled, col_empty, col_narrow, col_circle, col_linear] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ])
    .areas(area);

    render_square_filled_column(frame, col_filled, tick);
    render_square_empty_column(frame, col_empty, tick);
    render_narrow_column(frame, col_narrow, tick);
    render_circle_column(frame, col_circle, tick);
    render_linear_column(frame, col_linear, tick);
}

// ── Style table ───────────────────────────────────────────────────────────────

const RECT_STYLES: &[(RectStyle, Color, &str)] = &[
    (RectStyle::Arc, Color::Cyan, "Arc"),
    (RectStyle::Dense, Color::Magenta, "Dense"),
    (RectStyle::Shade, Color::Green, "Shade"),
    (RectStyle::Outline, Color::Yellow, "Outline"),
    (RectStyle::Dot, Color::White, "Dot"),
    (RectStyle::Star, Color::LightYellow, "Star"),
    (RectStyle::Diamond, Color::LightMagenta, "Diamond"),
    (RectStyle::Cross, Color::LightCyan, "Cross"),
    (RectStyle::Fade, Color::LightGreen, "Fade"),
    (RectStyle::Pixel, Color::LightRed, "Pixel"),
];

// ── Col 1 — Square, Filled centre ─────────────────────────────────────────────

fn render_square_filled_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" Square · Filled · ↻ ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let n = RECT_STYLES.len() as u16;
    let row_h = ((inner.height.saturating_sub(1)) / n).max(1);

    let mut constraints = vec![Constraint::Length(1)];
    for _ in RECT_STYLES {
        constraints.push(Constraint::Length(row_h));
    }
    constraints.push(Constraint::Min(0));
    let rows = Layout::vertical(constraints).split(inner);

    frame.render_widget(
        Paragraph::new(Span::styled(
            "size 2  size 3",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
        rows[0],
    );

    for (i, &(style, color, label)) in RECT_STYLES.iter().enumerate() {
        let row = rows[i + 1];
        let [s2, s3, lbl] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .areas(row);

        frame.render_widget(
            RectSpinner::new(tick)
                .shape(RectShape::Square(2))
                .render_style(style)
                .spin(Spin::Clockwise)
                .outer_color(color)
                .inner_color(Color::DarkGray)
                .centre(Centre::Filled)
                .ticks_per_step(3)
                .alignment(Alignment::Center),
            s2,
        );
        frame.render_widget(
            RectSpinner::new(tick)
                .shape(RectShape::Square(3))
                .render_style(style)
                .spin(Spin::Clockwise)
                .outer_color(color)
                .inner_color(Color::DarkGray)
                .centre(Centre::Filled)
                .ticks_per_step(5)
                .alignment(Alignment::Center),
            s3,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {label}"),
                Style::default().fg(color),
            )),
            lbl,
        );
    }
}

// ── Col 2 — Square, Empty centre ──────────────────────────────────────────────

fn render_square_empty_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" Square · Empty · ↺ ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightMagenta))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let n = RECT_STYLES.len() as u16;
    let row_h = ((inner.height.saturating_sub(1)) / n).max(1);

    let mut constraints = vec![Constraint::Length(1)];
    for _ in RECT_STYLES {
        constraints.push(Constraint::Length(row_h));
    }
    constraints.push(Constraint::Min(0));
    let rows = Layout::vertical(constraints).split(inner);

    frame.render_widget(
        Paragraph::new(Span::styled(
            "size 2  size 3",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
        rows[0],
    );

    for (i, &(style, color, label)) in RECT_STYLES.iter().enumerate() {
        let row = rows[i + 1];
        let [s2, s3, lbl] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .areas(row);

        frame.render_widget(
            RectSpinner::new(tick)
                .shape(RectShape::Square(2))
                .render_style(style)
                .spin(Spin::CounterClockwise)
                .outer_color(color)
                .centre(Centre::Empty)
                .ticks_per_step(3)
                .alignment(Alignment::Center),
            s2,
        );
        frame.render_widget(
            RectSpinner::new(tick)
                .shape(RectShape::Square(3))
                .render_style(style)
                .spin(Spin::CounterClockwise)
                .outer_color(color)
                .centre(Centre::Empty)
                .ticks_per_step(5)
                .alignment(Alignment::Center),
            s3,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {label}"),
                Style::default().fg(color),
            )),
            lbl,
        );
    }
}

// ── Col 3 — Narrow (1-char-wide Zed-style sidebar) ───────────────────────────
//
// Each row shows one Narrow spinner (1 char wide) in a different colour,
// all at the same arc speed.  The Narrow spinner ignores render_style and
// centre — it always renders the 10-frame Zed braille cap animation.

const NARROW_COLORS: &[(Color, &str)] = &[
    (Color::Cyan, "Cyan"),
    (Color::Magenta, "Magenta"),
    (Color::Green, "Green"),
    (Color::Yellow, "Yellow"),
    (Color::White, "White"),
    (Color::LightYellow, "Lt.Yellow"),
    (Color::LightMagenta, "Lt.Magenta"),
    (Color::LightCyan, "Lt.Cyan"),
    (Color::LightGreen, "Lt.Green"),
    (Color::LightRed, "Lt.Red"),
];

fn render_narrow_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" Narrow · Zed ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightGreen))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let n = NARROW_COLORS.len();
    let row_h = (inner.height / n as u16).max(1);

    let mut constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(row_h)).collect();
    constraints.push(Constraint::Min(0));
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(color, label)) in NARROW_COLORS.iter().enumerate() {
        let row = rows[i];
        let h = row_h.max(3) as usize;

        // Spinner occupies 1 char column; label gets the rest.
        let [spinner_area, lbl_area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            RectSpinner::new(tick)
                .shape(RectShape::Narrow(h))
                .outer_color(color)
                .inner_color(Color::DarkGray)
                .ticks_per_step(2),
            spinner_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {label}"),
                Style::default().fg(color),
            )),
            lbl_area,
        );
    }
}

// ── Col 4 — CircleSpinner ─────────────────────────────────────────────────────

fn render_circle_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" Circle ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightCyan))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // (radius, arc_color, dim_color, spin, label)
    let configs: &[(usize, Color, Color, Spin, &str)] = &[
        (2, Color::Cyan, Color::DarkGray, Spin::Clockwise, "r=2 ↻"),
        (
            3,
            Color::Magenta,
            Color::DarkGray,
            Spin::CounterClockwise,
            "r=3 ↺",
        ),
        (
            4,
            Color::LightGreen,
            Color::DarkGray,
            Spin::Clockwise,
            "r=4 ↻",
        ),
        (
            5,
            Color::LightYellow,
            Color::DarkGray,
            Spin::CounterClockwise,
            "r=5 ↺",
        ),
        (
            6,
            Color::LightRed,
            Color::DarkGray,
            Spin::Clockwise,
            "r=6 ↻",
        ),
        (
            8,
            Color::LightCyan,
            Color::DarkGray,
            Spin::CounterClockwise,
            "r=8 ↺",
        ),
    ];

    // Build row heights from the actual char_size of each spinner.
    let row_heights: Vec<u16> = configs
        .iter()
        .map(|&(r, .., _)| CircleSpinner::new(0).radius(r).char_size().1.max(1))
        .collect();

    let total_h: u16 = row_heights.iter().sum();
    let title_h = 1u16;

    let [title_area, body] =
        Layout::vertical([Constraint::Length(title_h), Constraint::Min(0)]).areas(inner);

    frame.render_widget(
        Paragraph::new(Span::styled("radius", Style::default().fg(Color::DarkGray)))
            .alignment(Alignment::Center),
        title_area,
    );

    // Use exact heights if they fit, otherwise distribute evenly.
    let constraints: Vec<Constraint> = if total_h <= body.height {
        row_heights
            .iter()
            .map(|&h| Constraint::Length(h))
            .chain(std::iter::once(Constraint::Min(0)))
            .collect()
    } else {
        configs
            .iter()
            .map(|_| Constraint::Ratio(1, configs.len() as u32))
            .collect()
    };

    let rows = Layout::vertical(constraints).split(body);

    for (i, &(radius, arc_col, dim_col, spin, label)) in configs.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let row = rows[i];

        let (spinner_w, _) = CircleSpinner::new(0).radius(radius).char_size();
        let [spinner_area, lbl_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            CircleSpinner::new(tick)
                .radius(radius)
                .spin(spin)
                .arc_color(arc_col)
                .dim_color(dim_col)
                .ticks_per_step(3),
            spinner_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {label}"),
                Style::default().fg(arc_col),
            )),
            lbl_area,
        );
    }
}

// ── Col 5 — LinearSpinner ─────────────────────────────────────────────────────

const DOT_STYLES: &[(LinearStyle, Color, &str)] = &[
    (LinearStyle::Classic, Color::White, "Classic  ●·"),
    (LinearStyle::Square, Color::Cyan, "Square   ■□"),
    (LinearStyle::Diamond, Color::Magenta, "Diamond  ◆◇"),
    (LinearStyle::Bar, Color::Green, "Bar      ▰▱"),
    (LinearStyle::Braille, Color::Yellow, "Braille  ⣿⠀"),
    (LinearStyle::Arrow, Color::LightRed, "Arrow    ▼▽"),
];

fn render_linear_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" LinearSpinner ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Split vertically: top half vertical bounce, bottom half horizontal scroll.
    let [vert_area, horiz_area] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(inner);

    render_vertical_dots(frame, vert_area, tick);
    render_horizontal_dots(frame, horiz_area, tick);
}

fn render_vertical_dots(frame: &mut Frame, area: Rect, tick: u64) {
    let [title, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Span::styled(
            "Vertical bounce",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
        title,
    );

    let row_h = (body.height / DOT_STYLES.len() as u16).max(1);
    let mut constraints: Vec<Constraint> = (0..DOT_STYLES.len())
        .map(|_| Constraint::Length(row_h))
        .collect();
    constraints.push(Constraint::Min(0));
    let rows = Layout::vertical(constraints).split(body);

    for (i, &(style, color, label)) in DOT_STYLES.iter().enumerate() {
        let row = rows[i];
        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Length(3), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            LinearSpinner::new(tick)
                .direction(Direction::Vertical)
                .linear_style(style)
                .total_slots(3)
                .ticks_per_step(6)
                .active_color(color)
                .inactive_color(Color::DarkGray),
            spinner_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {label}"),
                Style::default().fg(color),
            )),
            label_area,
        );
    }
}

fn render_horizontal_dots(frame: &mut Frame, area: Rect, tick: u64) {
    let [title, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(Span::styled(
            "Horizontal scroll",
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
        title,
    );

    let configs: &[(usize, usize, u64)] = &[
        (4, 1, 3),
        (4, 2, 3),
        (5, 2, 2),
        (5, 3, 2),
        (6, 2, 2),
        (6, 3, 1),
    ];

    let mut constraints: Vec<Constraint> = configs.iter().map(|_| Constraint::Length(1)).collect();
    constraints.push(Constraint::Min(0));
    let rows = Layout::vertical(constraints).split(body);

    for (i, (&(style, color, _), &(total, lit, tps))) in
        DOT_STYLES.iter().zip(configs.iter()).enumerate()
    {
        let row = rows[i];
        let spinner_w = (total + 1) as u16;
        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            LinearSpinner::new(tick)
                .direction(Direction::Horizontal)
                .linear_style(style)
                .total_slots(total)
                .lit_slots(lit)
                .ticks_per_step(tps * 2)
                .active_color(color)
                .inactive_color(Color::DarkGray),
            spinner_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {total}t {lit}lit"),
                Style::default().fg(Color::DarkGray),
            )),
            label_area,
        );
    }
}
