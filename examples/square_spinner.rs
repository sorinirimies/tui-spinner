//! # SquareSpinner Example
//!
//! Demonstrates [`SquareSpinner`] in all four combinations of
//! [`Centre`] (Filled / Empty) and [`Spin`] (Clockwise ↻ / Counter-Clockwise ↺).
//!
//! Layout — 2 × 2 grid:
//!
//! | | ↻ Clockwise | ↺ Counter-Clockwise |
//! |---|---|---|
//! | **Filled** | sizes 2 · 3 · 4 | sizes 2 · 3 · 4 |
//! | **Empty**  | sizes 2 · 3 · 4 | sizes 2 · 3 · 4 |
//!
//! **Controls:** `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example square_spinner`

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
use tui_spinner::{Centre, Spin, SquareSpinner};

// ── macros ────────────────────────────────────────────────────────────────────
// (sty!, sp!, section_block! — defined inline so the example is self-contained)
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

// ── App ───────────────────────────────────────────────────────────────────────
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

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, &mut App::default());
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        let steps = (Instant::now().duration_since(app.last_tick).as_millis() / 80).max(1) as u64;
        app.last_tick = Instant::now();
        app.tick = app.tick.wrapping_add(steps);
        terminal.draw(|f| render(f, app))?;
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(k) = event::read()? {
                if matches!(k.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }
    Ok(())
}

// ── Layout ────────────────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());
    render_header(frame, header);
    render_body(frame, body, app.tick);
    render_footer(frame, footer);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        sp!("Filled"; Color::Cyan),
        sp!("  ·  "; dim),
        sp!("Empty"; Color::Green),
        sp!("   ──   "; dim),
        sp!("↻ Clockwise"; Color::Yellow),
        sp!("  ·  "; dim),
        sp!("↺ Counter-Clockwise"; Color::Magenta),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .title(" SquareSpinner ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(sty!(dim)),
        ),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            sp!("q"; Color::Cyan, b),
            sp!(" / "; dim),
            sp!("Esc"; Color::Cyan, b),
            sp!("  Quit"; dim),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(sty!(dim)),
        ),
        area,
    );
}

fn render_body(frame: &mut Frame, area: Rect, tick: u64) {
    let [row_top, row_bot] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

    let [col_cw, col_ccw] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(row_top);
    render_quad(
        frame,
        col_cw,
        tick,
        Centre::Filled,
        Spin::Clockwise,
        "Filled · ↻ Clockwise",
        Color::Cyan,
    );
    render_quad(
        frame,
        col_ccw,
        tick,
        Centre::Filled,
        Spin::CounterClockwise,
        "Filled · ↺ Counter-CW",
        Color::Cyan,
    );

    let [col_cw2, col_ccw2] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(row_bot);
    render_quad(
        frame,
        col_cw2,
        tick,
        Centre::Empty,
        Spin::Clockwise,
        "Empty · ↻ Clockwise",
        Color::Green,
    );
    render_quad(
        frame,
        col_ccw2,
        tick,
        Centre::Empty,
        Spin::CounterClockwise,
        "Empty · ↺ Counter-CW",
        Color::Green,
    );
}

fn render_quad(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    centre: Centre,
    spin: Spin,
    title: &'static str,
    color: Color,
) {
    let accent = if matches!(spin, Spin::Clockwise) {
        color
    } else {
        Color::Magenta
    };
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(accent))
        .padding(Padding::uniform(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Label row
    let [label_row, spinner_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            sp!("size 2"; dim),
            sp!("          "; dim),
            sp!("size 3"; dim),
            sp!("          "; dim),
            sp!("size 4"; dim),
        ]))
        .alignment(Alignment::Center),
        label_row,
    );

    let [c1, c2, c3] = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .areas(spinner_area);

    for (area, sz) in [(c1, 2usize), (c2, 3), (c3, 4)] {
        frame.render_widget(
            SquareSpinner::new(tick)
                .size(sz)
                .spin(spin)
                .arc_color(color)
                .dim_color(Color::DarkGray)
                .centre(centre)
                .ticks_per_step(2 + sz as u64)
                .alignment(Alignment::Center),
            area,
        );
    }
}
