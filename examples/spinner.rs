//! # Spinner Widget Example
//!
//! Demonstrates all spinner widgets side-by-side:
//!
//! - **Col 1** — [`SquareSpinner`] filled centre, clockwise
//! - **Col 2** — [`SquareSpinner`] empty centre, counter-clockwise
//! - **Col 3** — [`CircleSpinner`] — various radii, CW and CCW
//! - **Col 4** — [`LinearSpinner`] vertical and horizontal
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
    BarMotion, BarSpinner, BarStyle, Centre, CircleSpinner, Direction, LinearSpinner, LinearStyle,
    Spin, SquareSpinner,
};

// ── Style macros ──────────────────────────────────────────────────────────────

/// Build a [`Style`] quickly.
macro_rules! sty {
    (dim) => {
        Style::default().fg(Color::DarkGray)
    };
    ($c:expr) => {
        Style::default().fg($c)
    };
    ($c:expr, b) => {
        Style::default().fg($c).add_modifier(Modifier::BOLD)
    };
}

/// Build a styled [`Span`] quickly.
macro_rules! sp {
    ($t:expr; dim) => {
        Span::styled($t, sty!(dim))
    };
    ($t:expr; $c:expr) => {
        Span::styled($t, sty!($c))
    };
    ($t:expr; $c:expr, b) => {
        Span::styled($t, sty!($c, b))
    };
}

/// Build a rounded bordered [`Block`] with a centred title and uniform padding.
macro_rules! section_block {
    ($title:expr, $color:expr) => {
        Block::bordered()
            .title(concat!(" ", $title, " "))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(sty!($color))
            .padding(Padding::uniform(1))
    };
}

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
    let [header, content, bar_row, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(5),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    render_header(frame, header);
    render_content(frame, content, app.tick);
    render_bar_row(frame, bar_row, app.tick);
    render_footer(frame, footer);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title(" tui-spinner Demo ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(Color::Cyan))
        .padding(Padding::horizontal(1));

    let text = Paragraph::new(
        "Square · Filled · CW  ·  Square · Empty · CCW  ·  Circle  ·  LinearSpinner  ·  BarSpinner",
    )
    .alignment(Alignment::Center)
    .style(sty!(Color::Gray));

    frame.render_widget(text.block(block), area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(sty!(dim))
        .padding(Padding::horizontal(1));

    let text = Line::from(vec![
        sp!("q"; Color::Cyan, b),
        Span::raw(" / "),
        sp!("Esc"; Color::Cyan, b),
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
    let [col_filled, col_empty, col_circle, col_linear] = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .areas(area);

    render_square_filled_column(frame, col_filled, tick);
    render_square_empty_column(frame, col_empty, tick);
    render_circle_column(frame, col_circle, tick);
    render_linear_column(frame, col_linear, tick);
}

// ── BarSpinner row ─────────────────────────────────────────────────────────────

fn render_bar_row(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" BarSpinner  ·  → Bounce  ←  ⟳ Loop ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(Color::LightGreen))
        .padding(Padding::horizontal(1));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Four representative styles side by side.
    // Each column shows one style with three bars: → Bounce, ← Bounce, ⟳ Loop.
    let styles: &[(BarStyle, Color, &str)] = &[
        (BarStyle::Braille, Color::Cyan, "Braille"),
        (BarStyle::Block, Color::LightGreen, "Block"),
        (BarStyle::Star, Color::Rgb(255, 220, 80), "Star"),
        (BarStyle::Progress, Color::LightBlue, "Progress"),
    ];

    let col_cs: Vec<Constraint> = (0..styles.len())
        .map(|_| Constraint::Ratio(1, styles.len() as u32))
        .collect();
    let cols = Layout::horizontal(col_cs).split(inner);

    for (i, &(style, color, _label)) in styles.iter().enumerate() {
        if i >= cols.len() {
            break;
        }
        let col = cols[i];

        let [r1, r2, r3] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(col);

        for (row, spin, motion, hint) in [
            (r1, Spin::Clockwise, BarMotion::Bounce, "→"),
            (r2, Spin::CounterClockwise, BarMotion::Bounce, "←"),
            (r3, Spin::Clockwise, BarMotion::Loop, "⟳"),
        ] {
            let [bar, hnt] =
                Layout::horizontal([Constraint::Min(4), Constraint::Length(2)]).areas(row);
            frame.render_widget(
                BarSpinner::new(tick)
                    .bar_style(style)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .spin(spin)
                    .motion(motion),
                bar,
            );
            frame.render_widget(
                Paragraph::new(Span::styled(hint, Style::default().fg(color))),
                hnt,
            );
        }
    }
}

// ── Col 1 — Square, Filled centre ─────────────────────────────────────────────

fn render_square_filled_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = section_block!("Square · Filled · CW", Color::Cyan);

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);

    frame.render_widget(
        Paragraph::new(sp!("size 2      size 3      size 4"; dim)).alignment(Alignment::Center),
        rows[0],
    );

    let cols = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(rows[1]);

    for (i, sz) in [2, 3, 4].iter().enumerate() {
        frame.render_widget(
            SquareSpinner::new(tick)
                .size(*sz)
                .spin(Spin::Clockwise)
                .arc_color(Color::Cyan)
                .dim_color(Color::DarkGray)
                .centre(Centre::Filled)
                .ticks_per_step(2 + *sz as u64)
                .alignment(Alignment::Center),
            cols[i],
        );
    }
}

// ── Col 2 — Square, Empty centre ──────────────────────────────────────────────

fn render_square_empty_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = section_block!("Square · Empty · CCW", Color::Green);

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(inner);

    frame.render_widget(
        Paragraph::new(sp!("size 2      size 3      size 4"; dim)).alignment(Alignment::Center),
        rows[0],
    );

    let cols = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(rows[1]);

    for (i, sz) in [2, 3, 4].iter().enumerate() {
        frame.render_widget(
            SquareSpinner::new(tick)
                .size(*sz)
                .spin(Spin::CounterClockwise)
                .arc_color(Color::Green)
                .centre(Centre::Empty)
                .ticks_per_step(2 + *sz as u64)
                .alignment(Alignment::Center),
            cols[i],
        );
    }
}

// ── Col 3 — Circle ────────────────────────────────────────────────────────────

fn render_circle_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = section_block!("Circle", Color::LightCyan);

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
        Paragraph::new(sp!("radius"; dim)).alignment(Alignment::Center),
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
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); arc_col)), lbl_area);
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
    let outer_block = section_block!("LinearSpinner", Color::Yellow);

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
        Paragraph::new(sp!("Vertical bounce"; dim)).alignment(Alignment::Center),
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
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); color)), label_area);
    }
}

fn render_horizontal_dots(frame: &mut Frame, area: Rect, tick: u64) {
    let [title, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    frame.render_widget(
        Paragraph::new(sp!("Horizontal scroll"; dim)).alignment(Alignment::Center),
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
            Paragraph::new(sp!(format!(" {total}t {lit}lit"); dim)),
            label_area,
        );
    }
}
