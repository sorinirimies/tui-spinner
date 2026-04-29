//! # BarSpinner Example
//!
//! Every style cell shows three bars: → Bounce, ← Bounce, ⟳ Loop.
//!
//! All 16 [`BarStyle`] variants are displayed in a 4×4 grid split into two
//! 2-row groups so the layout fills the terminal cleanly.
//!
//! **Controls:** `q` / `Esc` quit
//!
//! Run with: `cargo run --example bar_spinner`

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
    DefaultTerminal, Frame,
};
use std::time::{Duration, Instant};
use tui_spinner::{BarMotion, BarSpinner, BarStyle, Spin};

macro_rules! sty {
    (dim) => {
        Style::default().fg(Color::DarkGray)
    };
    ($c:expr) => {
        Style::default().fg($c)
    };
    ($c:expr,b) => {
        Style::default().fg($c).add_modifier(Modifier::BOLD)
    };
}
macro_rules! sp {
    ($t:expr;dim) => {
        Span::styled($t, sty!(dim))
    };
    ($t:expr;$c:expr) => {
        Span::styled($t, sty!($c))
    };
    ($t:expr;$c:expr,b) => {
        Span::styled($t, sty!($c, b))
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

    frame.render_widget(
        Paragraph::new(sp!("→ Bounce  ·  ← Bounce  ·  ⟳ Loop"; dim))
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title(" BarSpinner ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .border_style(sty!(dim)),
            ),
        header,
    );

    render_styles(frame, body, app.tick);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            sp!("q"; Color::Cyan, b),
            sp!(" / "; dim),
            sp!("Esc"; Color::Cyan, b),
            sp!("  quit"; dim),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(sty!(dim)),
        ),
        footer,
    );
}

// ── Style table ───────────────────────────────────────────────────────────────

/// All 16 [`BarStyle`] variants shown in a consistent 4×4 grid.
const STYLES: &[(BarStyle, &str, Color)] = &[
    // Group 1 — row 1
    (BarStyle::Braille, "Braille  ⣿⣀", Color::Cyan),
    (BarStyle::Block, "Block    █░", Color::LightGreen),
    (BarStyle::Shade, "Shade    ▓░", Color::LightCyan),
    (BarStyle::Dot, "Dot      ●·", Color::Yellow),
    // Group 1 — row 2
    (BarStyle::Diamond, "Diamond  ◆◇", Color::Magenta),
    (BarStyle::Square, "Square   ■□", Color::LightRed),
    (BarStyle::Star, "Star     ★☆", Color::Rgb(255, 220, 80)),
    (BarStyle::Heart, "Heart    ♥♡", Color::Rgb(255, 120, 160)),
    // Group 2 — row 1
    (BarStyle::Arrow, "Arrow    ▶▷", Color::Rgb(120, 200, 255)),
    (BarStyle::Circle, "Circle   ◉○", Color::Rgb(160, 255, 200)),
    (BarStyle::Spark, "Spark    ✦✧", Color::Rgb(255, 200, 120)),
    (BarStyle::Cross, "Cross    ✚✛", Color::Rgb(200, 160, 255)),
    // Group 2 — row 2
    (BarStyle::Progress, "Progress ▰▱", Color::Rgb(80, 220, 80)),
    (BarStyle::Thick, "Thick    ━─", Color::Rgb(200, 200, 200)),
    (BarStyle::Wave, "Wave     ≈˜", Color::Rgb(120, 200, 220)),
    (BarStyle::Pip, "Pip      ▪·", Color::Rgb(180, 180, 255)),
];

// ── Render ────────────────────────────────────────────────────────────────────

fn render_styles(frame: &mut Frame, area: Rect, tick: u64) {
    // Two groups of 2 rows (10 rows each) with a 1-row gap.
    let [g1, _gap, g2, _rest] = Layout::vertical([
        Constraint::Length(10),
        Constraint::Length(1),
        Constraint::Length(10),
        Constraint::Min(0),
    ])
    .areas(area);

    render_group(frame, g1, tick, &STYLES[..8]);
    render_group(frame, g2, tick, &STYLES[8..]);
}

/// Render up to 8 style entries in a 2-row × 4-col group.
fn render_group(frame: &mut Frame, area: Rect, tick: u64, items: &[(BarStyle, &str, Color)]) {
    let row_cs: Vec<Constraint> = (0..2)
        .map(|_| Constraint::Length(5))
        .chain([Constraint::Min(0)])
        .collect();
    let col_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let rows = Layout::vertical(row_cs).split(area);

    for (i, &(style, label, color)) in items.iter().enumerate() {
        let r = i / 4;
        let c = i % 4;
        if r >= rows.len().saturating_sub(1) {
            break;
        }
        let cols = Layout::horizontal(col_cs.clone()).split(rows[r]);
        cell(frame, cols[c], label, color, move |f, inner| {
            trio(f, inner, tick, color, move |t, s, m| {
                BarSpinner::new(t)
                    .bar_style(style)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .spin(s)
                    .motion(m)
            });
        });
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Three bars per cell: → Bounce, ← Bounce, ⟳ Loop.
fn trio<'a, F>(frame: &mut Frame, inner: Rect, tick: u64, color: Color, make: F)
where
    F: Fn(u64, Spin, BarMotion) -> BarSpinner<'a>,
{
    let [r1, r2, r3, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);

    for (row, spin, motion, hint) in [
        (r1, Spin::Clockwise, BarMotion::Bounce, "→"),
        (r2, Spin::CounterClockwise, BarMotion::Bounce, "←"),
        (r3, Spin::Clockwise, BarMotion::Loop, "⟳"),
    ] {
        let [bar, hnt] = Layout::horizontal([Constraint::Min(4), Constraint::Length(2)]).areas(row);
        frame.render_widget(make(tick, spin, motion), bar);
        frame.render_widget(Paragraph::new(sp!(hint.to_string(); color)), hnt);
    }
}

/// Compact bordered section.
fn cell<F>(frame: &mut Frame, area: Rect, title: &str, color: Color, f: F)
where
    F: FnOnce(&mut Frame, Rect),
{
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(color));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    f(frame, inner);
}
