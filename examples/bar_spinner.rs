//! # BarSpinner Example — direction + motion per style
//!
//! Every style cell shows three bars: → Bounce, ← Bounce, ⟳ Loop.
//!
//! Page 1: Mixed styles overview (4×4, two 2-row groups)
//! Page 2: Symbol Styles (4×4, two 2-row groups)
//! Page 3: Knobs (arc width, track, fade, arc char)
//!
//! Controls: ← / h  prev  ·  → / l  next  ·  q / Esc  quit
//!
//! Run with: cargo run --example bar_spinner

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
use tui_spinner::{BarMotion, BarSpinner, BarStyle, BarTrack, Spin};

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

const NUM_PAGES: usize = 3;
const PAGE_TITLES: [&str; 3] = ["Overview", "Symbol Styles", "Knobs"];

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
                    KeyCode::Left | KeyCode::Char('h') => app.page = app.page.saturating_sub(1),
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.page = (app.page + 1).min(NUM_PAGES - 1)
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let [hdr, body, ftr] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());
    render_header(frame, hdr, app.page);
    match app.page {
        0 => page_overview(frame, body, app.tick),
        1 => page_symbols(frame, body, app.tick),
        2 => page_knobs(frame, body, app.tick),
        _ => {}
    }
    render_footer(frame, ftr, app.page);
}

fn render_header(frame: &mut Frame, area: Rect, page: usize) {
    frame.render_widget(
        Paragraph::new(sp!(format!("{}/{} · {}", page+1, NUM_PAGES, PAGE_TITLES[page]); dim))
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title(" BarSpinner ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .border_style(sty!(dim)),
            ),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect, page: usize) {
    let prev = if page > 0 {
        "← / h  prev"
    } else {
        "             "
    };
    let next = if page + 1 < NUM_PAGES {
        "next  → / l"
    } else {
        "             "
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            sp!(prev; Color::Cyan),
            sp!("  "; dim),
            sp!("q"; Color::Cyan,b),
            sp!("/"; dim),
            sp!("Esc"; Color::Cyan,b),
            sp!(" quit"; dim),
            sp!("  "; dim),
            sp!(next; Color::Cyan),
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

// ── Trio helper — → Bounce, ← Bounce, ⟳ Loop ─────────────────────────────────
//
// Three bars in the cell's inner area, one per mode.  The hint character on
// the right indicates the direction / motion for that row.

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

// ── Cell helper ───────────────────────────────────────────────────────────────

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

/// Render up to 8 overview-style entries in a compact 2-row × 4-col group.
fn render_overview_group(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    items: &[(BarStyle, BarTrack, usize, usize, &str, Color)],
) {
    let row_cs: Vec<Constraint> = (0..2)
        .map(|_| Constraint::Length(5))
        .chain([Constraint::Min(0)])
        .collect();
    let col_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let rows = Layout::vertical(row_cs).split(area);

    for (i, &(style, track, height, fade, label, color)) in items.iter().enumerate() {
        let r = i / 4;
        let c = i % 4;
        if r >= rows.len().saturating_sub(1) {
            break;
        }
        let cols = Layout::horizontal(col_cs.clone()).split(rows[r]);
        cell(frame, cols[c], label, color, move |f, inner| {
            trio(f, inner, tick, color, move |t, s, m| {
                BarSpinner::new(t)
                    .height(height)
                    .bar_style(style)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .track(track)
                    .fade_width(fade)
                    .spin(s)
                    .motion(m)
            });
        });
    }
}

/// Render up to 8 symbol-style entries in a compact 2-row × 4-col group.
fn render_symbol_group(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    items: &[(BarStyle, &str, Color)],
) {
    let row_cs: Vec<Constraint> = (0..2)
        .map(|_| Constraint::Length(5))
        .chain([Constraint::Min(0)])
        .collect();
    let col_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let rows = Layout::vertical(row_cs).split(area);

    for (i, &(style, chars, color)) in items.iter().enumerate() {
        let r = i / 4;
        let c = i % 4;
        if r >= rows.len().saturating_sub(1) {
            break;
        }
        let cols = Layout::horizontal(col_cs.clone()).split(rows[r]);
        cell(frame, cols[c], chars, color, move |f, inner| {
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

// ── Page 1: Overview — tight 3×2 mixed grid ────────────────────────────────
//
// Six compact cells (Constraint::Length(5) = 2 borders + 3 trio bars).
// Mix of Braille and symbol styles so page 1 is immediately representative.

const OVERVIEW: &[(BarStyle, BarTrack, usize, usize, &str, Color)] = &[
    // (style, track, height, fade_width, label, color)
    // Row 1 — Braille variants
    (
        BarStyle::Braille,
        BarTrack::Rail,
        1,
        3,
        "Braille  Rail",
        Color::Cyan,
    ),
    (
        BarStyle::Braille,
        BarTrack::Full,
        1,
        0,
        "Braille  Full  0",
        Color::White,
    ),
    (
        BarStyle::Braille,
        BarTrack::Rail,
        2,
        3,
        "Braille  h=2",
        Color::LightBlue,
    ),
    (
        BarStyle::Braille,
        BarTrack::Empty,
        1,
        3,
        "Braille  Empty",
        Color::LightMagenta,
    ),
    // Row 2 — Block / geometric
    (
        BarStyle::Block,
        BarTrack::Rail,
        1,
        3,
        "Block    █░",
        Color::LightGreen,
    ),
    (
        BarStyle::Shade,
        BarTrack::Rail,
        1,
        3,
        "Shade    ▓░",
        Color::LightCyan,
    ),
    (
        BarStyle::Diamond,
        BarTrack::Rail,
        1,
        3,
        "Diamond  ◆◇",
        Color::Magenta,
    ),
    (
        BarStyle::Square,
        BarTrack::Rail,
        1,
        3,
        "Square   ■□",
        Color::LightRed,
    ),
    // Row 3 — Symbols / patterns  (group 2, row 1)
    (
        BarStyle::Star,
        BarTrack::Rail,
        1,
        3,
        "Star     ★☆",
        Color::Rgb(255, 220, 80),
    ),
    (
        BarStyle::Heart,
        BarTrack::Rail,
        1,
        3,
        "Heart    ♥♡",
        Color::Rgb(255, 120, 160),
    ),
    (
        BarStyle::Progress,
        BarTrack::Rail,
        1,
        3,
        "Progress ▰▱",
        Color::Rgb(80, 220, 80),
    ),
    (
        BarStyle::Wave,
        BarTrack::Rail,
        1,
        3,
        "Wave     ≈˜",
        Color::Rgb(120, 200, 220),
    ),
    // Row 4  (group 2, row 2)
    (
        BarStyle::Dot,
        BarTrack::Rail,
        1,
        3,
        "Dot      ●·",
        Color::Yellow,
    ),
    (
        BarStyle::Pip,
        BarTrack::Rail,
        1,
        3,
        "Pip      ▪·",
        Color::Rgb(180, 180, 255),
    ),
    (
        BarStyle::Arrow,
        BarTrack::Rail,
        1,
        3,
        "Arrow    ▶▷",
        Color::Rgb(120, 200, 255),
    ),
    (
        BarStyle::Circle,
        BarTrack::Rail,
        1,
        3,
        "Circle   ◉○",
        Color::Rgb(160, 255, 200),
    ),
];

fn page_overview(frame: &mut Frame, area: Rect, tick: u64) {
    // Two banks of 2 rows (10 rows each) separated by a 1-row gap.
    // Total: 10 + 1 + 10 = 21 rows, leaving a small margin at the bottom.
    let [g1, _gap, g2, _rest] = Layout::vertical([
        Constraint::Length(10),
        Constraint::Length(1),
        Constraint::Length(10),
        Constraint::Min(0),
    ])
    .areas(area);

    render_overview_group(frame, g1, tick, &OVERVIEW[..8]);
    render_overview_group(frame, g2, tick, &OVERVIEW[8..]);
}

// ── Page 2: Symbol Styles — 4×4 grid ─────────────────────────────────────────

const STYLES: &[(BarStyle, &str, Color)] = &[
    (BarStyle::Block, "█░", Color::LightGreen),
    (BarStyle::Shade, "▓░", Color::LightCyan),
    (BarStyle::Dot, "●·", Color::Yellow),
    (BarStyle::Diamond, "◆◇", Color::Magenta),
    (BarStyle::Square, "■□", Color::LightRed),
    (BarStyle::Star, "★☆", Color::Rgb(255, 220, 80)),
    (BarStyle::Heart, "♥♡", Color::Rgb(255, 120, 160)),
    (BarStyle::Arrow, "▶▷", Color::Rgb(120, 200, 255)),
    (BarStyle::Circle, "◉○", Color::Rgb(160, 255, 200)),
    (BarStyle::Spark, "✦✧", Color::Rgb(255, 200, 120)),
    (BarStyle::Cross, "✚✛", Color::Rgb(200, 160, 255)),
    (BarStyle::Progress, "▰▱", Color::Rgb(80, 220, 80)),
    (BarStyle::Thick, "━─", Color::Rgb(200, 200, 200)),
    (BarStyle::Wave, "≈˜", Color::Rgb(120, 200, 220)),
    (BarStyle::Pip, "▪·", Color::Rgb(180, 180, 255)),
    (BarStyle::Braille, "⣿⣀", Color::Cyan),
];

fn page_symbols(frame: &mut Frame, area: Rect, tick: u64) {
    // Same 2-bank layout as page_overview.
    let [g1, _gap, g2, _rest] = Layout::vertical([
        Constraint::Length(10),
        Constraint::Length(1),
        Constraint::Length(10),
        Constraint::Min(0),
    ])
    .areas(area);

    render_symbol_group(frame, g1, tick, &STYLES[..8]);
    render_symbol_group(frame, g2, tick, &STYLES[8..]);
}

// ── Page 3: Knobs — 2×2 grid ─────────────────────────────────────────────────

fn page_knobs(frame: &mut Frame, area: Rect, tick: u64) {
    let [top, bot] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [tl, tr] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);
    let [bl, br] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(bot);

    cell(
        frame,
        tl,
        "arc_width  narrow (5)  vs  wide (20)",
        Color::LightRed,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            knob_trio(f, ha, tick, "arc=5", Color::LightRed, |t, s, m| {
                BarSpinner::new(t)
                    .arc_width(5)
                    .arc_color(Color::LightRed)
                    .dim_color(Color::DarkGray)
                    .spin(s)
                    .motion(m)
            });
            knob_trio(f, hb, tick, "arc=20", Color::LightRed, |t, s, m| {
                BarSpinner::new(t)
                    .arc_width(20)
                    .arc_color(Color::LightRed)
                    .dim_color(Color::DarkGray)
                    .spin(s)
                    .motion(m)
            });
        },
    );
    cell(
        frame,
        tr,
        "track  Rail  vs  Empty",
        Color::LightYellow,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            knob_trio(f, ha, tick, "Rail", Color::LightYellow, |t, s, m| {
                BarSpinner::new(t)
                    .track(BarTrack::Rail)
                    .arc_color(Color::LightYellow)
                    .dim_color(Color::DarkGray)
                    .spin(s)
                    .motion(m)
            });
            knob_trio(
                f,
                hb,
                tick,
                "Empty (floats)",
                Color::LightYellow,
                |t, s, m| {
                    BarSpinner::new(t)
                        .track(BarTrack::Empty)
                        .arc_color(Color::LightYellow)
                        .dim_color(Color::Black)
                        .spin(s)
                        .motion(m)
                },
            );
        },
    );
    cell(
        frame,
        bl,
        "fade_width  sharp (0)  vs  soft (3)",
        Color::LightMagenta,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            knob_trio(
                f,
                ha,
                tick,
                "fade=0 sharp",
                Color::LightMagenta,
                |t, s, m| {
                    BarSpinner::new(t)
                        .fade_width(0)
                        .arc_color(Color::LightMagenta)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                        .motion(m)
                },
            );
            knob_trio(
                f,
                hb,
                tick,
                "fade=3 soft",
                Color::LightMagenta,
                |t, s, m| {
                    BarSpinner::new(t)
                        .fade_width(3)
                        .arc_color(Color::LightMagenta)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                        .motion(m)
                },
            );
        },
    );
    cell(
        frame,
        br,
        "arc_char  full (0xFF)  vs  light (0x3F)",
        Color::LightCyan,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            knob_trio(f, ha, tick, "0xFF ⣿ full", Color::LightCyan, |t, s, m| {
                BarSpinner::new(t)
                    .arc_char(0xFF)
                    .arc_color(Color::LightCyan)
                    .dim_color(Color::DarkGray)
                    .spin(s)
                    .motion(m)
            });
            knob_trio(
                f,
                hb,
                tick,
                "0x3F ⠿ light",
                Color::LightCyan,
                |t, s, m| {
                    BarSpinner::new(t)
                        .arc_char(0x3F)
                        .arc_color(Color::LightCyan)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                        .motion(m)
                },
            );
        },
    );
}

// A labeled group of 3 bars (trio) with a dim label row above.
fn knob_trio<'a, F>(frame: &mut Frame, area: Rect, tick: u64, label: &str, color: Color, make: F)
where
    F: Fn(u64, Spin, BarMotion) -> BarSpinner<'a>,
{
    let [lbl, body, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(0),
    ])
    .areas(area);
    frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), lbl);
    trio(frame, body, tick, color, make);
}
