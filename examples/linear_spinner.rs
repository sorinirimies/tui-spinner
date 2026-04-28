//! # LinearSpinner Example
//!
//! Demonstrates [`LinearSpinner`] across all combinations of
//! [`Direction`] (Horizontal / Vertical) and [`Flow`] (Forwards / Backwards),
//! for every [`LinearStyle`] variant.
//!
//! Layout:
//! - **Top half** — Horizontal orientation
//!   - Forwards ↻: all 6 styles
//!   - Backwards ↺: all 6 styles
//! - **Bottom half** — Vertical orientation
//!   - Left column: Forwards ↻  (all 6 styles)
//!   - Right column: Backwards ↺ (all 6 styles)
//!
//! **Controls:** `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example linear_spinner`

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
use tui_spinner::{Direction, Flow, LinearSpinner, LinearStyle};

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

// ── Style configs ─────────────────────────────────────────────────────────────
/// (style, active_color, label)
const STYLES: &[(LinearStyle, Color, &str)] = &[
    (LinearStyle::Classic, Color::White, "Classic   ●·"),
    (LinearStyle::Square, Color::Cyan, "Square    ■□"),
    (LinearStyle::Diamond, Color::Magenta, "Diamond   ◆◇"),
    (LinearStyle::Bar, Color::LightGreen, "Bar       ▰▱"),
    (LinearStyle::Braille, Color::Yellow, "Braille   ⣿⠀"),
    (LinearStyle::Arrow, Color::LightRed, "Arrow     ▼▽"),
];

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
        sp!("Horizontal"; Color::Cyan),
        sp!("  ·  "; dim),
        sp!("Vertical"; Color::LightGreen),
        sp!("   ──   "; dim),
        sp!("↻ Forwards"; Color::Yellow),
        sp!("  ·  "; dim),
        sp!("↺ Backwards"; Color::Magenta),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .title(" LinearSpinner ")
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
    let [top, bottom] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    render_horizontal(frame, top, tick);
    render_vertical(frame, bottom, tick);
}

// ── Horizontal ────────────────────────────────────────────────────────────────

fn render_horizontal(frame: &mut Frame, area: Rect, tick: u64) {
    let block = section_block!("Horizontal", Color::Cyan);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [fwd, bwd] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(inner);
    render_horiz_flow(
        frame,
        fwd,
        tick,
        Flow::Forwards,
        "↻  Forwards",
        Color::Yellow,
    );
    render_horiz_flow(
        frame,
        bwd,
        tick,
        Flow::Backwards,
        "↺  Backwards",
        Color::Magenta,
    );
}

fn render_horiz_flow(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    flow: Flow,
    title: &str,
    color: Color,
) {
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded)
        .border_style(sty!(color))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let n = STYLES.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(style, active_color, label)) in STYLES.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let [spin_area, lbl_area] =
            Layout::horizontal([Constraint::Length(6), Constraint::Min(0)]).areas(rows[i]);
        frame.render_widget(
            LinearSpinner::new(tick)
                .direction(Direction::Horizontal)
                .linear_style(style)
                .flow(flow)
                .total_slots(5)
                .lit_slots(2)
                .ticks_per_step(3)
                .active_color(active_color)
                .inactive_color(Color::DarkGray),
            spin_area,
        );
        frame.render_widget(
            Paragraph::new(sp!(format!("  {label}"); active_color)),
            lbl_area,
        );
    }
}

// ── Vertical ──────────────────────────────────────────────────────────────────

fn render_vertical(frame: &mut Frame, area: Rect, tick: u64) {
    let block = section_block!("Vertical", Color::LightGreen);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [col_fwd, col_bwd] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(inner);
    render_vert_flow(
        frame,
        col_fwd,
        tick,
        Flow::Forwards,
        "↻  Forwards",
        Color::Yellow,
    );
    render_vert_flow(
        frame,
        col_bwd,
        tick,
        Flow::Backwards,
        "↺  Backwards",
        Color::Magenta,
    );
}

fn render_vert_flow(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    flow: Flow,
    title: &str,
    color: Color,
) {
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded)
        .border_style(sty!(color))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let n = STYLES.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(style, active_color, label)) in STYLES.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let [spin_area, lbl_area] =
            Layout::horizontal([Constraint::Length(3), Constraint::Min(0)]).areas(rows[i]);
        frame.render_widget(
            LinearSpinner::new(tick)
                .direction(Direction::Vertical)
                .linear_style(style)
                .flow(flow)
                .total_slots(3)
                .ticks_per_step(5)
                .active_color(active_color)
                .inactive_color(Color::DarkGray),
            spin_area,
        );
        frame.render_widget(
            Paragraph::new(sp!(format!("  {label}"); active_color)),
            lbl_area,
        );
    }
}
