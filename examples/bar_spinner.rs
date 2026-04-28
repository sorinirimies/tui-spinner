//! # BarSpinner Example
//!
//! Three-page interactive demo — every concept shows **exactly one Clockwise ↻
//! bar and one Counter-Clockwise ↺ bar** so the difference is always obvious.
//!
//! | Page | Focus |
//! |------|-------|
//! | 1 | Heights & styles — different visual patterns |
//! | 2 | Style knobs — arc_width, track, fade_width, arc_char |
//! | 3 | Presets — [`BarSpinner::zed`] etc. |
//!
//! **Controls:** `←` / `h` prev  ·  `→` / `l` next  ·  `q` / `Esc` quit
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
use tui_spinner::{BarSpinner, BarTrack, Spin};

// ── Style macros ──────────────────────────────────────────────────────────────

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

// ── Pages ─────────────────────────────────────────────────────────────────────

const NUM_PAGES: usize = 3;
const PAGE_TITLES: [&str; 3] = ["Heights & Styles", "Style Knobs", "Presets"];

// ── App ───────────────────────────────────────────────────────────────────────

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

// ── Shell ─────────────────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());
    render_header(frame, header, app.page);
    match app.page {
        0 => page_heights(frame, body, app.tick),
        1 => page_knobs(frame, body, app.tick),
        2 => page_presets(frame, body, app.tick),
        _ => {}
    }
    render_footer(frame, footer, app.page);
}

fn render_header(frame: &mut Frame, area: Rect, page: usize) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            sp!(format!("{}/{}  ·  {}", page + 1, NUM_PAGES, PAGE_TITLES[page]); dim),
        ]))
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
        "            "
    };
    let next = if page + 1 < NUM_PAGES {
        "next  → / l"
    } else {
        "            "
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            sp!(prev; Color::Cyan),
            sp!("     "; dim),
            sp!("q"; Color::Cyan, b),
            sp!(" / "; dim),
            sp!("Esc"; Color::Cyan, b),
            sp!("  quit"; dim),
            sp!("     "; dim),
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

// ── Pair helper ───────────────────────────────────────────────────────────────
//
// Renders a labelled concept as exactly TWO bars: one CW ↻, one CCW ↺.
// `make` receives the direction and returns the configured spinner.

fn render_pair<'a, F>(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    label: &str,
    height: usize,
    color: Color,
    make: F,
) where
    F: Fn(u64, Spin) -> BarSpinner<'a>,
{
    // Vertical split: 1-row label  +  height CW  +  height CCW  +  leftover.
    let constraints = vec![
        Constraint::Length(1),
        Constraint::Length(height as u16),
        Constraint::Length(height as u16),
        Constraint::Min(0),
    ];
    let rows = Layout::vertical(constraints).split(area);

    // Label row
    frame.render_widget(
        Paragraph::new(Line::from(vec![sp!(format!(" {label}"); color, b)])),
        rows[0],
    );

    // CW bar + direction hint
    render_bar_with_hint(
        frame,
        rows[1],
        tick,
        color,
        make(tick, Spin::Clockwise),
        "↻",
    );
    // CCW bar + direction hint
    render_bar_with_hint(
        frame,
        rows[2],
        tick,
        color,
        make(tick, Spin::CounterClockwise),
        "↺",
    );
}

fn render_bar_with_hint<'a>(
    frame: &mut Frame,
    row: Rect,
    _tick: u64,
    color: Color,
    spinner: BarSpinner<'a>,
    hint: &str,
) {
    // Reserve 2 chars on the right for the ↻/↺ hint.
    let hint_w = 2u16;
    let bar_w = row.width.saturating_sub(hint_w);
    let [bar_area, hint_area] =
        Layout::horizontal([Constraint::Length(bar_w), Constraint::Length(hint_w)]).areas(row);

    frame.render_widget(spinner, bar_area);
    frame.render_widget(Paragraph::new(sp!(hint.to_string(); color)), hint_area);
}

// ── Page 1 — Heights & Styles ─────────────────────────────────────────────────
//
// Four visually distinct pattern concepts, each shown as 1 CW + 1 CCW bar.

fn page_heights(frame: &mut Frame, area: Rect, tick: u64) {
    // Each concept: (label, height, label_color, CW+CCW builder)
    // We allocate 1 label row + 2*height content rows per concept.
    let concepts: &[(
        &str,
        usize,
        Color,
        Box<dyn Fn(u64, Spin) -> BarSpinner<'static>>,
    )] = &[
        // 1 — Thin Zed-style, cyan, Rail track
        (
            "Zed-style  ·  h=1  ·  cyan  ·  Rail track",
            1,
            Color::Cyan,
            Box::new(|t, s| {
                BarSpinner::new(t)
                    .height(1)
                    .arc_color(Color::Cyan)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            }),
        ),
        // 2 — Claude 2-row, warm orange, Rail track
        (
            "Claude-style  ·  h=2  ·  orange  ·  Rail track",
            2,
            Color::Rgb(255, 165, 0),
            Box::new(|t, s| {
                BarSpinner::new(t)
                    .height(2)
                    .arc_color(Color::Rgb(255, 165, 0))
                    .dim_color(Color::DarkGray)
                    .spin(s)
            }),
        ),
        // 3 — Sharp edges, no fade, white arc, Full track
        (
            "Sharp  ·  h=1  ·  white  ·  Full track  ·  fade=0",
            1,
            Color::White,
            Box::new(|t, s| {
                BarSpinner::new(t)
                    .height(1)
                    .arc_color(Color::White)
                    .dim_color(Color::DarkGray)
                    .track(BarTrack::Full)
                    .fade_width(0)
                    .spin(s)
            }),
        ),
        // 4 — Floating arc, Empty track (arc on empty space)
        (
            "Float  ·  h=1  ·  magenta  ·  Empty track",
            1,
            Color::LightMagenta,
            Box::new(|t, s| {
                BarSpinner::new(t)
                    .height(1)
                    .arc_color(Color::LightMagenta)
                    .dim_color(Color::Black)
                    .track(BarTrack::Empty)
                    .spin(s)
            }),
        ),
    ];

    // Build height-aware constraints: 1 label + 2*h content + 1 gap, per concept.
    let per_concept: Vec<u16> = concepts
        .iter()
        .map(|(_, h, ..)| 1 + 2 * *h as u16 + 1)
        .collect();
    let constraints: Vec<Constraint> = per_concept
        .iter()
        .map(|&h| Constraint::Length(h))
        .chain([Constraint::Min(0)])
        .collect();
    let slots = Layout::vertical(constraints).split(area);

    for (i, (label, height, color, make)) in concepts.iter().enumerate() {
        if i >= slots.len().saturating_sub(1) {
            break;
        }
        render_pair(frame, slots[i], tick, label, *height, *color, |t, s| {
            make(t, s)
        });
    }
}

// ── Page 2 — Style Knobs ──────────────────────────────────────────────────────
//
// 2×2 grid. Each cell demonstrates one style dimension with a CW+CCW pair.

fn page_knobs(frame: &mut Frame, area: Rect, tick: u64) {
    let [top, bot] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [tl, tr] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);
    let [bl, br] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(bot);

    knob_cell(
        frame,
        tl,
        tick,
        "Arc width  narrow (arc=5) vs wide (arc=20)",
        Color::LightRed,
        |frame, inner, tick, _title, color| {
            let [top_half, bot_half] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            render_pair(
                frame,
                top_half,
                tick,
                "arc_width = 5   (narrow)",
                1,
                color,
                |t, s| {
                    BarSpinner::new(t)
                        .arc_width(5)
                        .arc_color(color)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
            render_pair(
                frame,
                bot_half,
                tick,
                "arc_width = 20  (wide)",
                1,
                color,
                |t, s| {
                    BarSpinner::new(t)
                        .arc_width(20)
                        .arc_color(color)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
        },
    );

    knob_cell(
        frame,
        tr,
        tick,
        "Track  Rail vs Empty",
        Color::LightYellow,
        |frame, inner, tick, _title, color| {
            let [top_half, bot_half] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            render_pair(
                frame,
                top_half,
                tick,
                "track = Rail  (⣀ baseline)",
                1,
                color,
                |t, s| {
                    BarSpinner::new(t)
                        .track(BarTrack::Rail)
                        .arc_color(color)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
            render_pair(
                frame,
                bot_half,
                tick,
                "track = Empty  (arc floats)",
                1,
                color,
                |t, s| {
                    BarSpinner::new(t)
                        .track(BarTrack::Empty)
                        .arc_color(color)
                        .dim_color(Color::Black)
                        .spin(s)
                },
            );
        },
    );

    knob_cell(
        frame,
        bl,
        tick,
        "Fade width  sharp vs soft",
        Color::LightMagenta,
        |frame, inner, tick, _title, color| {
            let [top_half, bot_half] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            render_pair(
                frame,
                top_half,
                tick,
                "fade_width = 0  (sharp ⣿)",
                1,
                color,
                |t, s| {
                    BarSpinner::new(t)
                        .fade_width(0)
                        .arc_color(color)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
            render_pair(
                frame,
                bot_half,
                tick,
                "fade_width = 3  (soft ⠉⠛⠿⣿)",
                1,
                color,
                |t, s| {
                    BarSpinner::new(t)
                        .fade_width(3)
                        .arc_color(color)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
        },
    );

    knob_cell(
        frame,
        br,
        tick,
        "Arc char  full vs light",
        Color::LightCyan,
        |frame, inner, tick, _title, color| {
            let [top_half, bot_half] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            render_pair(
                frame,
                top_half,
                tick,
                "arc_char = 0xFF  ⣿  full",
                1,
                color,
                |t, s| {
                    BarSpinner::new(t)
                        .arc_char(0xFF)
                        .arc_color(color)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
            render_pair(
                frame,
                bot_half,
                tick,
                "arc_char = 0x3F  ⠿  light",
                1,
                color,
                |t, s| {
                    BarSpinner::new(t)
                        .arc_char(0x3F)
                        .arc_color(color)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
        },
    );
}

/// Draw a rounded titled cell and call `inner` with the inner area.
fn knob_cell<F>(frame: &mut Frame, area: Rect, tick: u64, title: &str, color: Color, inner_fn: F)
where
    F: FnOnce(&mut Frame, Rect, u64, &str, Color),
{
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(color));
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    inner_fn(frame, inner_area, tick, title, color);
}

// ── Page 3 — Presets ──────────────────────────────────────────────────────────

fn page_presets(frame: &mut Frame, area: Rect, tick: u64) {
    let block = Block::bordered()
        .title(" Presets — BarSpinner::name(tick) ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(Color::White));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    type CtorFn = fn(u64) -> BarSpinner<'static>;
    let presets: &[(CtorFn, &str, usize, Color)] = &[
        (
            BarSpinner::zed,
            "BarSpinner::zed(tick)     ·  1 row  ·  cyan  ·  Rail",
            1,
            Color::Cyan,
        ),
        (
            BarSpinner::claude,
            "BarSpinner::claude(tick)  ·  2 rows ·  orange · Rail",
            2,
            Color::Rgb(255, 165, 0),
        ),
        (
            BarSpinner::minimal,
            "BarSpinner::minimal(tick) ·  1 row  ·  white ·  Empty (arc floats)",
            1,
            Color::White,
        ),
        (
            BarSpinner::solid,
            "BarSpinner::solid(tick)   ·  1 row  ·  cyan  ·  Full · fade=0",
            1,
            Color::Cyan,
        ),
    ];

    let per: Vec<u16> = presets
        .iter()
        .map(|&(_, _, h, _)| 1 + 2 * h as u16 + 1)
        .collect();
    let constraints: Vec<Constraint> = per
        .iter()
        .map(|&h| Constraint::Length(h))
        .chain([Constraint::Min(0)])
        .collect();
    let slots = Layout::vertical(constraints).split(inner);

    for (i, &(ctor, label, height, color)) in presets.iter().enumerate() {
        if i >= slots.len().saturating_sub(1) {
            break;
        }
        render_pair(frame, slots[i], tick, label, height, color, |t, s| {
            ctor(t).spin(s)
        });
    }
}
