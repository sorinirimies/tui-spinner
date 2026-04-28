//! # CircleSpinner Example
//!
//! Demonstrates [`CircleSpinner`] in both rotation directions across six radii.
//!
//! - **Left column**  — Clockwise ↻
//! - **Right column** — Counter-Clockwise ↺
//!
//! Each row shows the spinner at that radius next to a `r=N` label.
//!
//! **Controls:** `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example circle_spinner`

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
use tui_spinner::{CircleSpinner, Spin};

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

// ── Radii and colours ─────────────────────────────────────────────────────────
/// (radius, arc_color)
const RADII: &[(usize, Color)] = &[
    (2, Color::Cyan),
    (3, Color::LightBlue),
    (4, Color::Magenta),
    (5, Color::LightMagenta),
    (6, Color::Yellow),
    (8, Color::LightYellow),
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
        sp!("↻ Clockwise"; Color::Cyan),
        sp!("   ·   radius 2 → 3 → 4 → 5 → 6 → 8   ·   "; dim),
        sp!("Counter-Clockwise ↺"; Color::Magenta),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .title(" CircleSpinner ")
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
    let [col_cw, col_ccw] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    render_direction_col(
        frame,
        col_cw,
        tick,
        Spin::Clockwise,
        "↻  Clockwise",
        Color::Cyan,
    );
    render_direction_col(
        frame,
        col_ccw,
        tick,
        Spin::CounterClockwise,
        "↺  Counter-Clockwise",
        Color::Magenta,
    );
}

fn render_direction_col(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    spin: Spin,
    title: &str,
    color: Color,
) {
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(color))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let n = RADII.len();
    let row_constraints: Vec<Constraint> = RADII
        .iter()
        .map(|&(r, _)| {
            let h = CircleSpinner::new(0).radius(r).char_size().1.max(1);
            Constraint::Length(h)
        })
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(row_constraints).split(inner);

    for (i, &(radius, arc_color)) in RADII.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let spinner_w = CircleSpinner::new(0).radius(radius).char_size().0;

        let [spin_area, lbl_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            CircleSpinner::new(tick)
                .radius(radius)
                .spin(spin)
                .arc_color(arc_color)
                .dim_color(Color::DarkGray)
                .ticks_per_step(3),
            spin_area,
        );
        frame.render_widget(
            Paragraph::new(sp!(format!("  r={radius}  "); arc_color)),
            lbl_area,
        );
    }
    // suppress unused-variable warning for n
    let _ = n;
}
