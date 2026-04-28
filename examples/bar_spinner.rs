//! # BarSpinner Example
//!
//! Three-page interactive demo of [`BarSpinner`].
//!
//! | Page | Content |
//! |------|---------|
//! | 1 | Heights & directions — 1/2/3-row bars, CW ↻ vs CCW ↺ |
//! | 2 | Style knobs — arc width, track, fade, arc_char |
//! | 3 | Presets — [`BarSpinner::zed`], [`BarSpinner::claude`], etc. |
//!
//! **Controls:** `←` / `→` change page  ·  `q` / `Esc` quit
//!
//! Run with: `cargo run --example bar_spinner`

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

macro_rules! section_block {
    ($title:expr, $color:expr) => {
        Block::bordered()
            .title(concat!(" ", $title, " "))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(sty!($color))
            .padding(Padding::horizontal(1))
    };
}

// ── Page constants ────────────────────────────────────────────────────────────

const NUM_PAGES: usize = 3;

const PAGE_TITLES: [&str; NUM_PAGES] = ["Heights & Directions", "Style Knobs", "Presets"];

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

// ── Entry point ───────────────────────────────────────────────────────────────

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

// ── Root layout ───────────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    render_header(frame, header, app.page);
    render_page(frame, body, app.tick, app.page);
    render_footer(frame, footer, app.page);
}

fn render_header(frame: &mut Frame, area: Rect, page: usize) {
    let title = PAGE_TITLES[page];
    frame.render_widget(
        Paragraph::new(sp!(format!("Page {} of {}  ·  {}", page + 1, NUM_PAGES, title); dim))
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
        "           "
    };
    let next = if page + 1 < NUM_PAGES {
        "next  → / l"
    } else {
        "           "
    };
    let line = Line::from(vec![
        sp!(prev; Color::Cyan),
        sp!("     "; dim),
        sp!("q"; Color::Cyan, b),
        sp!(" / "; dim),
        sp!("Esc"; Color::Cyan, b),
        sp!("  Quit"; dim),
        sp!("     "; dim),
        sp!(next; Color::Cyan),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(sty!(dim)),
        ),
        area,
    );
}

fn render_page(frame: &mut Frame, area: Rect, tick: u64, page: usize) {
    match page {
        0 => render_page_heights(frame, area, tick),
        1 => render_page_knobs(frame, area, tick),
        2 => render_page_presets(frame, area, tick),
        _ => {}
    }
}

// ── Page 1: Heights & Directions ─────────────────────────────────────────────

fn render_page_heights(frame: &mut Frame, area: Rect, tick: u64) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area);
    render_heights_section(frame, left, tick);
    render_directions_section(frame, right, tick);
}

// ── height configs: (height_rows, arc_color, dim_color, spin, tps, label)
const HEIGHT_ROWS: &[(usize, Color, Color, Spin, u64, &str)] = &[
    // 1-row Zed-style
    (
        1,
        Color::Cyan,
        Color::DarkGray,
        Spin::Clockwise,
        1,
        "h=1  ↻  cyan",
    ),
    (
        1,
        Color::LightBlue,
        Color::DarkGray,
        Spin::CounterClockwise,
        1,
        "h=1  ↺  blue",
    ),
    (
        1,
        Color::White,
        Color::DarkGray,
        Spin::Clockwise,
        2,
        "h=1  ↻  slow",
    ),
    (
        1,
        Color::Cyan,
        Color::Black,
        Spin::Clockwise,
        1,
        "h=1  ↻  no track",
    ),
    // 2-row Claude-style
    (
        2,
        Color::Yellow,
        Color::DarkGray,
        Spin::Clockwise,
        1,
        "h=2  ↻  yellow",
    ),
    (
        2,
        Color::Rgb(255, 165, 0),
        Color::DarkGray,
        Spin::CounterClockwise,
        2,
        "h=2  ↺  orange",
    ),
    (
        2,
        Color::Rgb(255, 200, 50),
        Color::Rgb(60, 30, 0),
        Spin::Clockwise,
        1,
        "h=2  ↻  warm bg",
    ),
    // 3-row thick
    (
        3,
        Color::LightGreen,
        Color::DarkGray,
        Spin::Clockwise,
        1,
        "h=3  ↻  green",
    ),
    (
        3,
        Color::Magenta,
        Color::DarkGray,
        Spin::CounterClockwise,
        2,
        "h=3  ↺  slow",
    ),
];

fn render_heights_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Heights  1 · 2 · 3 rows", Color::Cyan);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // One row per config; height of each row = the spinner's row count.
    let constraints: Vec<Constraint> = HEIGHT_ROWS
        .iter()
        .map(|&(h, ..)| Constraint::Length(h as u16))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(h, arc, dim, spin, tps, label)) in HEIGHT_ROWS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let label_w = (label.len() as u16 + 2).min(rows[i].width.saturating_sub(4));
        let spinner_w = rows[i].width.saturating_sub(label_w);
        let [spin_area, lbl_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)])
                .areas(rows[i]);

        frame.render_widget(
            BarSpinner::new(tick)
                .height(h)
                .arc_color(arc)
                .dim_color(dim)
                .spin(spin)
                .ticks_per_step(tps),
            spin_area,
        );
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); arc)), lbl_area);
    }
}

// ── CW vs CCW: (color, tps)
const DIR_PAIRS: &[(Color, u64)] = &[
    (Color::Cyan, 1),
    (Color::Yellow, 1),
    (Color::Magenta, 2),
    (Color::LightGreen, 1),
];

fn render_directions_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("CW ↻  vs  CCW ↺  ·  same tick", Color::LightCyan);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = DIR_PAIRS.len();
    let pair_h = (inner.height / n as u16).max(2);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(pair_h))
        .chain([Constraint::Min(0)])
        .collect();
    let slots = Layout::vertical(constraints).split(inner);

    for (i, &(color, tps)) in DIR_PAIRS.iter().enumerate() {
        if i >= slots.len().saturating_sub(1) {
            break;
        }
        let slot = slots[i];
        let [cw_row, ccw_row] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(slot);

        for (row, spin) in [(cw_row, Spin::Clockwise), (ccw_row, Spin::CounterClockwise)] {
            let arrow = if matches!(spin, Spin::Clockwise) {
                "↻"
            } else {
                "↺"
            };
            let label_w = 4u16;
            let spinner_w = row.width.saturating_sub(label_w);
            let [spin_area, lbl_area] =
                Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)])
                    .areas(row);

            frame.render_widget(
                BarSpinner::new(tick)
                    .height(1)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .spin(spin)
                    .ticks_per_step(tps),
                spin_area,
            );
            frame.render_widget(Paragraph::new(sp!(format!(" {arrow}  "); color)), lbl_area);
        }
    }
}

// ── Page 2: Style Knobs ───────────────────────────────────────────────────────

fn render_page_knobs(frame: &mut Frame, area: Rect, tick: u64) {
    let [top, bottom] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [tl, tr] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);
    let [bl, br] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(bottom);

    render_arc_width_section(frame, tl, tick);
    render_track_section(frame, tr, tick);
    render_fade_section(frame, bl, tick);
    render_arc_char_section(frame, br, tick);
}

// ── arc width
const ARC_WIDTHS: &[(usize, &str)] = &[
    (3, "arc=3   narrow"),
    (6, "arc=6"),
    (10, "arc=10  wide"),
    (0, "arc=auto"),
];

fn render_arc_width_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Arc width  .arc_width(n)", Color::LightRed);
    render_knob_rows(frame, outer, area, tick, |frame, i, row, tick| {
        let &(arc_w, label) = &ARC_WIDTHS[i];
        let label_w = (label.len() as u16 + 2).min(row.width.saturating_sub(4));
        let spinner_w = row.width.saturating_sub(label_w);
        let [spin_area, lbl_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)])
                .areas(row);
        frame.render_widget(
            BarSpinner::new(tick)
                .arc_width(arc_w)
                .arc_color(Color::LightRed)
                .dim_color(Color::DarkGray),
            spin_area,
        );
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), lbl_area);
    });
}

// ── track style
const TRACK_CONFIGS: &[(BarTrack, &str)] = &[
    (BarTrack::Rail, "Rail  ⣀  default"),
    (BarTrack::Full, "Full  ⣿"),
    (BarTrack::Empty, "Empty ⠀  float"),
    (BarTrack::Custom(0x09), "Custom 0x09  ⠉"),
];

fn render_track_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Track  .track(BarTrack::…)", Color::LightYellow);
    render_knob_rows(frame, outer, area, tick, |frame, i, row, tick| {
        let &(track, label) = &TRACK_CONFIGS[i];
        let label_w = (label.len() as u16 + 2).min(row.width.saturating_sub(4));
        let spinner_w = row.width.saturating_sub(label_w);
        let [spin_area, lbl_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)])
                .areas(row);
        frame.render_widget(
            BarSpinner::new(tick)
                .track(track)
                .arc_color(Color::Yellow)
                .dim_color(Color::DarkGray),
            spin_area,
        );
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), lbl_area);
    });
}

// ── fade width
const FADE_CONFIGS: &[(usize, &str)] = &[
    (0, "fade=0  sharp  ⣿"),
    (1, "fade=1  subtle"),
    (2, "fade=2"),
    (3, "fade=3  default  ⠉⠛⠿⣿"),
];

fn render_fade_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Fade  .fade_width(n)", Color::LightMagenta);
    render_knob_rows(frame, outer, area, tick, |frame, i, row, tick| {
        let &(fw, label) = &FADE_CONFIGS[i];
        let label_w = (label.len() as u16 + 2).min(row.width.saturating_sub(4));
        let spinner_w = row.width.saturating_sub(label_w);
        let [spin_area, lbl_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)])
                .areas(row);
        frame.render_widget(
            BarSpinner::new(tick)
                .fade_width(fw)
                .arc_color(Color::Magenta)
                .dim_color(Color::DarkGray),
            spin_area,
        );
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), lbl_area);
    });
}

// ── arc char
const ARC_CHAR_CONFIGS: &[(u8, &str)] = &[
    (0xFF, "0xFF  ⣿  full (default)"),
    (0x7F, "0x7F  ⡿  7 dots"),
    (0x3F, "0x3F  ⠿  6 dots"),
    (0x1B, "0x1B  ⠛  4 dots"),
];

fn render_arc_char_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Arc char  .arc_char(byte)", Color::LightCyan);
    render_knob_rows(frame, outer, area, tick, |frame, i, row, tick| {
        let &(byte, label) = &ARC_CHAR_CONFIGS[i];
        let label_w = (label.len() as u16 + 2).min(row.width.saturating_sub(4));
        let spinner_w = row.width.saturating_sub(label_w);
        let [spin_area, lbl_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)])
                .areas(row);
        frame.render_widget(
            BarSpinner::new(tick)
                .arc_char(byte)
                .arc_color(Color::Cyan)
                .dim_color(Color::DarkGray),
            spin_area,
        );
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), lbl_area);
    });
}

/// Generic helper: draw a section block and allocate N equal rows for a
/// caller-supplied render closure.  All four style-knob sections share this.
fn render_knob_rows<F>(
    frame: &mut Frame,
    block: Block<'_>,
    area: Rect,
    tick: u64,
    mut render_row: F,
) where
    F: FnMut(&mut Frame, usize, Rect, u64),
{
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let n = 4usize;
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for i in 0..n {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        render_row(frame, i, rows[i], tick);
    }
}

// ── Page 3: Presets ───────────────────────────────────────────────────────────

fn render_page_presets(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Presets — BarSpinner::preset(tick)", Color::White);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    struct Preset {
        label: &'static str,
        desc: &'static str,
        height: usize,
        color: Color,
    }

    let presets: &[(fn(u64) -> BarSpinner<'static>, Preset)] = &[
        (
            BarSpinner::zed,
            Preset {
                label: "BarSpinner::zed(tick)",
                desc: "1 row · cyan · Rail track · default settings",
                height: 1,
                color: Color::Cyan,
            },
        ),
        (
            BarSpinner::claude,
            Preset {
                label: "BarSpinner::claude(tick)",
                desc: "2 rows · orange · Rail track",
                height: 2,
                color: Color::Rgb(255, 165, 0),
            },
        ),
        (
            BarSpinner::minimal,
            Preset {
                label: "BarSpinner::minimal(tick)",
                desc: "1 row · white · Empty track — arc floats",
                height: 1,
                color: Color::White,
            },
        ),
        (
            BarSpinner::solid,
            Preset {
                label: "BarSpinner::solid(tick)",
                desc: "1 row · cyan · Full track · sharp edges",
                height: 1,
                color: Color::Cyan,
            },
        ),
    ];

    // Layout: for each preset, a name+desc header row + the spinner itself.
    let row_unit = 1u16; // 1 header + height rows + 1 gap
    let constraints: Vec<Constraint> = presets
        .iter()
        .flat_map(|(_, p)| {
            [
                Constraint::Length(1),               // label
                Constraint::Length(p.height as u16), // spinner
                Constraint::Length(1),               // gap
            ]
        })
        .chain([Constraint::Min(0)])
        .collect();
    let _ = row_unit;
    let rows = Layout::vertical(constraints).split(inner);

    for (i, (ctor, p)) in presets.iter().enumerate() {
        let base = i * 3;
        if base + 1 >= rows.len() {
            break;
        }
        let lbl_row = rows[base];
        let spin_row = rows[base + 1];

        // Label row: "BarSpinner::zed(tick)  ·  desc"
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                sp!(p.label; p.color, b),
                sp!("  ·  "; dim),
                sp!(p.desc; dim),
            ])),
            lbl_row,
        );

        // Spinner row: full width
        frame.render_widget(ctor(tick), spin_row);
    }
}
