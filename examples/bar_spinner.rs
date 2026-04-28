//! # BarSpinner Example — 16 symbol styles, 3 pages
//!
//! Page 1: Braille (2×2 sections)
//! Page 2: Symbol Styles (4×4 grid, 16 variants)
//! Page 3: Knobs
//!
//! Controls: <- / h prev  -> / l next  q / Esc quit

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
const PAGE_TITLES: [&str; 3] = ["Braille", "Symbol Styles", "Knobs"];

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

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, &mut App::default());
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> color_eyre::Result<()> {
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
        0 => page_braille(frame, body, app.tick),
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
        "<- / h  prev"
    } else {
        "             "
    };
    let next = if page + 1 < NUM_PAGES {
        "next  -> / l"
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

// ── Pair helper — 1 CW + 1 CCW bar ───────────────────────────────────────────

fn pair<'a, F>(frame: &mut Frame, inner: Rect, tick: u64, h: usize, color: Color, make: F)
where
    F: Fn(u64, Spin) -> BarSpinner<'a>,
{
    let [cw, ccw, _] = Layout::vertical([
        Constraint::Length(h as u16),
        Constraint::Length(h as u16),
        Constraint::Min(0),
    ])
    .areas(inner);
    for (area, spin, hint) in [
        (cw, Spin::Clockwise, "↻"),
        (ccw, Spin::CounterClockwise, "↺"),
    ] {
        let [bar, hnt] =
            Layout::horizontal([Constraint::Min(4), Constraint::Length(2)]).areas(area);
        frame.render_widget(make(tick, spin), bar);
        frame.render_widget(Paragraph::new(sp!(hint.to_string(); color)), hnt);
    }
}

// ── Cell helper — compact bordered section ────────────────────────────────────

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

// ── Page 1: Braille ───────────────────────────────────────────────────────────

fn page_braille(frame: &mut Frame, area: Rect, tick: u64) {
    let [top, bot] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [tl, tr] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);
    let [bl, br] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(bot);

    cell(
        frame,
        tl,
        "Bounce  h=1 · Rail  (ping-pong)",
        Color::Cyan,
        |f, i| {
            pair(f, i, tick, 1, Color::Cyan, |t, s| {
                BarSpinner::new(t)
                    .height(1)
                    .arc_color(Color::Cyan)
                    .dim_color(Color::DarkGray)
                    .motion(BarMotion::Bounce)
                    .spin(s)
            });
        },
    );
    cell(
        frame,
        tr,
        "Loop  h=1 · Rail  (continuous sweep)",
        Color::LightGreen,
        |f, i| {
            pair(f, i, tick, 1, Color::LightGreen, |t, s| {
                BarSpinner::new(t)
                    .height(1)
                    .arc_color(Color::LightGreen)
                    .dim_color(Color::DarkGray)
                    .motion(BarMotion::Loop)
                    .spin(s)
            });
        },
    );
    cell(
        frame,
        bl,
        "h=1 · Full track · fade=0  (sharp)",
        Color::White,
        |f, i| {
            pair(f, i, tick, 1, Color::White, |t, s| {
                BarSpinner::new(t)
                    .height(1)
                    .arc_color(Color::White)
                    .dim_color(Color::DarkGray)
                    .track(BarTrack::Full)
                    .fade_width(0)
                    .spin(s)
            });
        },
    );
    cell(
        frame,
        br,
        "h=2 · Rail · two rows",
        Color::LightBlue,
        |f, i| {
            pair(f, i, tick, 2, Color::LightBlue, |t, s| {
                BarSpinner::new(t)
                    .height(2)
                    .arc_color(Color::LightBlue)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        },
    );
}

// ── Page 2: Symbol Styles 4x4 ────────────────────────────────────────────────

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
    let row_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let col_cs: Vec<Constraint> = (0..4).map(|_| Constraint::Ratio(1, 4)).collect();
    let rows = Layout::vertical(row_cs).split(area);

    for (i, &(style, chars, color)) in STYLES.iter().enumerate() {
        let r = i / 4;
        let c = i % 4;
        if r >= rows.len() {
            break;
        }
        let cols = Layout::horizontal(col_cs.clone()).split(rows[r]);
        cell(frame, cols[c], chars, color, move |f, inner| {
            pair(f, inner, tick, 1, color, move |t, s| {
                BarSpinner::new(t)
                    .bar_style(style)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        });
    }
}

// ── Page 3: Knobs ─────────────────────────────────────────────────────────────

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
        "arc_width narrow vs wide",
        Color::LightRed,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            lpair(f, ha, tick, "arc=5", 1, Color::LightRed, |t, s| {
                BarSpinner::new(t)
                    .arc_width(5)
                    .arc_color(Color::LightRed)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
            lpair(f, hb, tick, "arc=20", 1, Color::LightRed, |t, s| {
                BarSpinner::new(t)
                    .arc_width(20)
                    .arc_color(Color::LightRed)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        },
    );
    cell(
        frame,
        tr,
        "track Rail vs Empty",
        Color::LightYellow,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            lpair(f, ha, tick, "Rail", 1, Color::LightYellow, |t, s| {
                BarSpinner::new(t)
                    .track(BarTrack::Rail)
                    .arc_color(Color::LightYellow)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
            lpair(
                f,
                hb,
                tick,
                "Empty (floats)",
                1,
                Color::LightYellow,
                |t, s| {
                    BarSpinner::new(t)
                        .track(BarTrack::Empty)
                        .arc_color(Color::LightYellow)
                        .dim_color(Color::Black)
                        .spin(s)
                },
            );
        },
    );
    cell(
        frame,
        bl,
        "fade_width sharp vs soft",
        Color::LightMagenta,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            lpair(f, ha, tick, "fade=0", 1, Color::LightMagenta, |t, s| {
                BarSpinner::new(t)
                    .fade_width(0)
                    .arc_color(Color::LightMagenta)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
            lpair(f, hb, tick, "fade=3", 1, Color::LightMagenta, |t, s| {
                BarSpinner::new(t)
                    .fade_width(3)
                    .arc_color(Color::LightMagenta)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        },
    );
    cell(
        frame,
        br,
        "arc_char full vs light",
        Color::LightCyan,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            lpair(f, ha, tick, "0xFF ⣿", 1, Color::LightCyan, |t, s| {
                BarSpinner::new(t)
                    .arc_char(0xFF)
                    .arc_color(Color::LightCyan)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
            lpair(f, hb, tick, "0x3F ⠿", 1, Color::LightCyan, |t, s| {
                BarSpinner::new(t)
                    .arc_char(0x3F)
                    .arc_color(Color::LightCyan)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        },
    );
}

fn lpair<'a, F>(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    label: &str,
    h: usize,
    color: Color,
    make: F,
) where
    F: Fn(u64, Spin) -> BarSpinner<'a>,
{
    let [lbl, body, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(h as u16 * 2),
        Constraint::Min(0),
    ])
    .areas(area);
    frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), lbl);
    pair(frame, body, tick, h, color, make);
}
