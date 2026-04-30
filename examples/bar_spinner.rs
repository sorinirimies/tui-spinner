//! # BarSpinner Example
//!
//! **Page 1** — Horizontal: all 16 [`BarStyle`] variants with 5 motions.
//!
//! **Page 2** — Vertical: same 16 styles, all 5 motions.
//!
//! **Page 3** — Thickness: `.thickness(1..4)` for both horizontal and vertical bars.
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

const NUM_PAGES: usize = 3;

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
            "↓ Bounce · ↑ Bounce · ⟳ Loop · ⟷ Squeeze · ✦ Radiate (Vertical)",
            render_vertical,
        ),
        2 => (
            ".thickness(1) · .thickness(2) · .thickness(3) · .thickness(4)",
            render_thickness,
        ),
        _ => (
            "→ Bounce · ← Bounce · ⟳ Loop · ⟷ Squeeze · ✦ Radiate (Horizontal)",
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
    let [r1, _g1, r2, _g2, r3, _g3, r4] = Layout::vertical([
        Constraint::Fill(1),   // row 1
        Constraint::Length(1), // gap
        Constraint::Fill(1),   // row 2
        Constraint::Length(1), // gap
        Constraint::Fill(1),   // row 3
        Constraint::Length(1), // gap
        Constraint::Fill(1),   // row 4
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

/// Five bars per cell showing all motions — with explanation labels.
fn trio<'a, F>(frame: &mut Frame, inner: Rect, tick: u64, color: Color, make: F)
where
    F: Fn(u64, Spin, BarMotion) -> BarSpinner<'a>,
{
    let [r1, _g1, r2, _g2, r3, _g3, r4, _g4, r5] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);

    for (row, spin, motion, hint) in [
        (r1, Spin::Clockwise, BarMotion::Bounce, "→ Bounce"),
        (r2, Spin::CounterClockwise, BarMotion::Bounce, "← Bounce"),
        (r3, Spin::Clockwise, BarMotion::Loop, "⟳ Loop"),
        (r4, Spin::Clockwise, BarMotion::Squeeze, "⟷ Squeeze"),
        (r5, Spin::Clockwise, BarMotion::Radiate, "✦ Radiate"),
    ] {
        let [bar, hnt] =
            Layout::horizontal([Constraint::Min(4), Constraint::Length(11)]).areas(row);
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

// ── Page 2: Vertical bars ───────────────────────────────────────────────────────────────
//
// Same 4×4 cell grid as horizontal.  Each cell contains three vertical
// spinners side-by-side: ↓ Bounce, ↑ Bounce, ⟳ Loop — with a hint label
// at the bottom of each column.

fn render_vertical(frame: &mut Frame, area: Rect, tick: u64) {
    // Ratio rows so cells fill the terminal height (unlike Length(5) for horizontal).
    let row_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let col_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let rows = Layout::vertical(row_cs).split(area);

    for (i, &(style, label, color)) in STYLES.iter().enumerate() {
        let r = i / 4;
        let c = i % 4;
        if r >= rows.len() {
            break;
        }
        let cols = Layout::horizontal(col_cs.clone()).split(rows[r]);

        // Bordered cell — same style as horizontal page.
        let block = Block::bordered()
            .title(format!(" {label} "))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(sty!(color));
        let inner = block.inner(cols[c]);
        frame.render_widget(block, cols[c]);

        // Five columns inside the cell: all motion types.
        // Each spinner is narrow (width 1) with a label below,
        // evenly spaced across the cell.
        let [c1, c2, c3, c4, c5] = Layout::horizontal([
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
            Constraint::Ratio(1, 5),
        ])
        .areas(inner);

        for (col_area, spin, motion, hint) in [
            (c1, Spin::Clockwise, BarMotion::Bounce, "↓"),
            (c2, Spin::CounterClockwise, BarMotion::Bounce, "↑"),
            (c3, Spin::Clockwise, BarMotion::Loop, "⟳"),
            (c4, Spin::Clockwise, BarMotion::Squeeze, "⟷"),
            (c5, Spin::Clockwise, BarMotion::Radiate, "✦"),
        ] {
            // Spinner fills the area minus a 1-row hint at the bottom.
            // Constrain the spinner to width 1 and center it.
            let [bar_area, hint_area] =
                Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(col_area);

            // Center a narrow spinner (width 1) within the column.
            let [_pad_l, narrow_bar, _pad_r] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(bar_area);

            frame.render_widget(
                BarSpinner::new(tick)
                    .orientation(BarOrientation::Vertical)
                    .bar_style(style)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .spin(spin)
                    .motion(motion)
                    .ticks_per_step(3),
                narrow_bar,
            );
            frame.render_widget(
                Paragraph::new(sp!(hint.to_string(); color)).alignment(Alignment::Center),
                hint_area,
            );
        }
    }
}

// ── Page 3: Thickness variations ────────────────────────────────────────────────────────
//
// Shows both horizontal and vertical bars at thickness 1, 2, 3, 4 side-by-side
// so users can see the effect of `.thickness()`.

/// A subset of styles to showcase thickness (4 styles × 4 thicknesses per orientation).
/// Multi-row/column bars automatically use cell-filling block characters.
const THICKNESS_STYLES: &[(BarStyle, &str, Color)] = &[
    (BarStyle::Shade, "Shade", Color::LightCyan),
    (BarStyle::Block, "Block", Color::LightGreen),
    (BarStyle::Diamond, "Diamond", Color::Magenta),
    (BarStyle::Progress, "Progress", Color::Rgb(80, 220, 80)),
];

fn render_thickness(frame: &mut Frame, area: Rect, tick: u64) {
    // Split into top (horizontal) and bottom (vertical) halves.
    let [top, _gap, bottom] = Layout::vertical([
        Constraint::Percentage(50),
        Constraint::Length(1),
        Constraint::Percentage(50),
    ])
    .areas(area);

    render_thickness_horizontal(frame, top, tick);
    render_thickness_vertical(frame, bottom, tick);
}

fn render_thickness_horizontal(frame: &mut Frame, area: Rect, tick: u64) {
    let block = Block::bordered()
        .title(" Horizontal — .thickness(n) ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // 4 columns, one per style.
    let col_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let cols = Layout::horizontal(col_cs).split(inner);

    for (c, &(style, name, color)) in THICKNESS_STYLES.iter().enumerate() {
        if c >= cols.len() {
            break;
        }

        let col_block = Block::bordered()
            .title(format!(" {name} "))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(sty!(color));
        let col_inner = col_block.inner(cols[c]);
        frame.render_widget(col_block, cols[c]);

        // Pack bars tightly: wrap them in a fixed-height container so the
        // layout solver can't distribute extra space between Length constraints.
        // Total = 1 + 1(gap) + 2 + 1(gap) + 3 + 1(gap) + 4 = 13
        let [bars_area, _rest] =
            Layout::vertical([Constraint::Length(13), Constraint::Min(0)]).areas(col_inner);

        let [r1, _g1, r2, _g2, r3, _g3, r4] = Layout::vertical([
            Constraint::Length(1), // thickness 1
            Constraint::Length(1), // gap
            Constraint::Length(2), // thickness 2
            Constraint::Length(1), // gap
            Constraint::Length(3), // thickness 3
            Constraint::Length(1), // gap
            Constraint::Length(4), // thickness 4
        ])
        .areas(bars_area);

        for (row_area, thickness) in [(r1, 1), (r2, 2), (r3, 3), (r4, 4)] {
            // Split: label on right, bar on left.
            let [bar, lbl] =
                Layout::horizontal([Constraint::Min(4), Constraint::Length(5)]).areas(row_area);
            frame.render_widget(
                BarSpinner::new(tick)
                    .bar_style(style)
                    .thickness(thickness)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .motion(BarMotion::Bounce),
                bar,
            );
            frame.render_widget(Paragraph::new(sp!(format!(" t={thickness}"); color)), lbl);
        }
    }
}

fn render_thickness_vertical(frame: &mut Frame, area: Rect, tick: u64) {
    let block = Block::bordered()
        .title(" Vertical — .thickness(n) ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(Color::LightGreen));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // 4 columns, one per style.
    let col_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let cols = Layout::horizontal(col_cs).split(inner);

    for (c, &(style, name, color)) in THICKNESS_STYLES.iter().enumerate() {
        if c >= cols.len() {
            break;
        }

        let col_block = Block::bordered()
            .title(format!(" {name} "))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(sty!(color));
        let col_inner = col_block.inner(cols[c]);
        frame.render_widget(col_block, cols[c]);

        // 4 sub-columns: thickness 1, 2, 3, 4 — side by side with labels below.
        let sub_cols = Layout::horizontal([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(col_inner);

        for (i, thickness) in [1usize, 2, 3, 4].iter().enumerate() {
            if i >= sub_cols.len() {
                break;
            }
            let [bar_area, lbl_area] =
                Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(sub_cols[i]);

            // Center the spinner at the specified thickness.
            let spinner_width = *thickness;
            let [_pad_l, narrow, _pad_r] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(spinner_width as u16),
                Constraint::Fill(1),
            ])
            .areas(bar_area);

            frame.render_widget(
                BarSpinner::new(tick)
                    .orientation(BarOrientation::Vertical)
                    .bar_style(style)
                    .thickness(*thickness)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .motion(BarMotion::Bounce)
                    .ticks_per_step(3),
                narrow,
            );
            frame.render_widget(
                Paragraph::new(sp!(format!("t={thickness}"); color)).alignment(Alignment::Center),
                lbl_area,
            );
        }
    }
}
