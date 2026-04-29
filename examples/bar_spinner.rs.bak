//! # BarSpinner Example
//!
//! **Page 1** — Horizontal: all 16 [`BarStyle`] variants with → Bounce, ← Bounce, ⟳ Loop.
//!
//! **Page 2** — Vertical: same 16 styles, arc bounces top ↕ bottom.
//!
//! **Controls:** `←`/`h` prev · `→`/`l` next · `q`/`Esc` quit
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
use tui_spinner::{BarMotion, BarOrientation, BarSpinner, BarStyle, Spin};

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

const NUM_PAGES: usize = 2;

struct App {
    tick: u64,
    last_tick: Instant,
    page: usize,
}
impl Default for App {
    fn default() -> Self {
        Self {
            tick: 0,
            last_tick: Instant::now(),
            page: 0,
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
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.page = app.page.saturating_sub(1);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.page = (app.page + 1).min(NUM_PAGES - 1);
                    }
                    _ => {}
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

    let margin = |area: Rect| -> Rect {
        let [_l, inner, _r] = Layout::horizontal([
            Constraint::Percentage(8),
            Constraint::Percentage(84),
            Constraint::Percentage(8),
        ])
        .areas(area);
        inner
    };

    let (subtitle, page_fn): (&str, fn(&mut Frame, Rect, u64)) = match app.page {
        1 => (
            "↕ Bounce  ·  ↕ Bounce  ·  ⟳ Loop  (Vertical)",
            render_vertical,
        ),
        _ => (
            "→ Bounce  ·  ← Bounce  ·  ⟳ Loop  (Horizontal)",
            render_styles,
        ),
    };

    let prev = if app.page > 0 {
        "← / h  prev"
    } else {
        "            "
    };
    let next = if app.page + 1 < NUM_PAGES {
        "next  → / l"
    } else {
        "            "
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            sp!(format!("{}/{} · ", app.page + 1, NUM_PAGES); dim),
            sp!(subtitle; dim),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" BarSpinner ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(sty!(dim)),
        ),
        margin(header),
    );

    page_fn(frame, margin(body), app.tick);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            sp!(prev; Color::Cyan),
            sp!("  "; dim),
            sp!("q"; Color::Cyan, b),
            sp!(" / "; dim),
            sp!("Esc"; Color::Cyan, b),
            sp!("  quit"; dim),
            sp!("  "; dim),
            sp!(next; Color::Cyan),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(sty!(dim)),
        ),
        margin(footer),
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
    // Horizontal margins make each of the 4 columns ~21% wide instead of 25%.
    let [_l, inner, _r] = Layout::horizontal([
        Constraint::Percentage(8),
        Constraint::Percentage(84),
        Constraint::Percentage(8),
    ])
    .areas(area);

    // Four rows separated by equal gaps — fills the vertical space evenly.
    let [r1, _g1, r2, _g2, r3, _g3, r4, _rest] = Layout::vertical([
        Constraint::Length(5), // row 1
        Constraint::Length(1), // gap
        Constraint::Length(5), // row 2
        Constraint::Length(1), // gap
        Constraint::Length(5), // row 3
        Constraint::Length(1), // gap
        Constraint::Length(5), // row 4
        Constraint::Min(0),
    ])
    .areas(inner);

    for (row_area, slice) in [
        (r1, &STYLES[0..4]),
        (r2, &STYLES[4..8]),
        (r3, &STYLES[8..12]),
        (r4, &STYLES[12..16]),
    ] {
        render_group(frame, row_area, tick, slice);
    }
}

/// Render one row of up to 4 style entries across 4 equal columns.
fn render_group(frame: &mut Frame, area: Rect, tick: u64, items: &[(BarStyle, &str, Color)]) {
    let col_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let cols = Layout::horizontal(col_cs).split(area);

    for (c, &(style, label, color)) in items.iter().enumerate() {
        if c >= cols.len() {
            break;
        }
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

// ── Page 2: Vertical bars ────────────────────────────────────────────────────
//
// Same 16 styles, Vertical orientation.  Each column is a narrow tall spinner;
// the arc bounces top↕bottom (even indices) or loops continuously (odd).

fn render_vertical(frame: &mut Frame, area: Rect, tick: u64) {
    // 16 columns of col_w chars each, centred in the available area.
    let col_w = 5u16;
    let n = STYLES.len() as u16;
    let total_w = col_w * n;
    let side = area.width.saturating_sub(total_w) / 2;

    let [_l, inner, _r] = Layout::horizontal([
        Constraint::Length(side),
        Constraint::Length(total_w),
        Constraint::Min(0),
    ])
    .areas(area);

    let col_cs: Vec<Constraint> = (0..n).map(|_| Constraint::Length(col_w)).collect();
    let cols = Layout::horizontal(col_cs).split(inner);

    for (i, &(style, label, color)) in STYLES.iter().enumerate() {
        if i >= cols.len() {
            break;
        }
        let col = cols[i];
        let title = label
            .chars()
            .take((col_w as usize).saturating_sub(2))
            .collect::<String>();
        let block = Block::bordered()
            .title(format!(" {} ", title.trim()))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(sty!(color));
        let inner_col = block.inner(col);
        frame.render_widget(block, col);

        let motion = if i % 2 == 0 {
            BarMotion::Bounce
        } else {
            BarMotion::Loop
        };
        frame.render_widget(
            BarSpinner::new(tick)
                .orientation(BarOrientation::Vertical)
                .bar_style(style)
                .arc_color(color)
                .dim_color(Color::DarkGray)
                .motion(motion)
                .ticks_per_step(3),
            inner_col,
        );
    }
}
