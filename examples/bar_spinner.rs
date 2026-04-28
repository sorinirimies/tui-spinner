//! # BarSpinner Example
//!
//! Three-page interactive demo organised into bordered sections.
//! Every concept shows exactly one Clockwise and one Counter-Clockwise bar.
//!
//! Page 1: Braille style (heights, track modes, fade)
//! Page 2: Symbol styles (Block, Shade, Dot, Diamond, Square)
//! Page 3: Knobs (arc width, track, fade, arc char)
//!
//! Controls: <- / h prev  ->  / l next  q / Esc quit
//!
//! Run with: cargo run --example bar_spinner

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
use tui_spinner::{BarSpinner, BarStyle, BarTrack, Spin};

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
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());
    render_header(frame, header, app.page);
    match app.page {
        0 => page_braille(frame, body, app.tick),
        1 => page_symbols(frame, body, app.tick),
        2 => page_knobs(frame, body, app.tick),
        _ => {}
    }
    render_footer(frame, footer, app.page);
}

fn render_header(frame: &mut Frame, area: Rect, page: usize) {
    frame.render_widget(
        Paragraph::new(sp!(format!("{}/{}  .  {}", page+1, NUM_PAGES, PAGE_TITLES[page]); dim))
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

fn render_pair_in<'a, F>(
    frame: &mut Frame,
    inner: Rect,
    tick: u64,
    height: usize,
    color: Color,
    make: F,
) where
    F: Fn(u64, Spin) -> BarSpinner<'a>,
{
    let [cw_area, ccw_area, _] = Layout::vertical([
        Constraint::Length(height as u16),
        Constraint::Length(height as u16),
        Constraint::Min(0),
    ])
    .areas(inner);
    let hint_w = 2u16;
    for (area, spin, hint) in [
        (cw_area, Spin::Clockwise, "↻"),
        (ccw_area, Spin::CounterClockwise, "↺"),
    ] {
        let [bar_area, hint_area] =
            Layout::horizontal([Constraint::Min(4), Constraint::Length(hint_w)]).areas(area);
        frame.render_widget(make(tick, spin), bar_area);
        frame.render_widget(Paragraph::new(sp!(hint.to_string(); color)), hint_area);
    }
}

fn section<F>(frame: &mut Frame, area: Rect, title: &str, color: Color, render: F)
where
    F: FnOnce(&mut Frame, Rect),
{
    let block = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(color))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    render(frame, inner);
}

fn page_braille(frame: &mut Frame, area: Rect, tick: u64) {
    let [top, bot] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [tl, tr] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);
    let [bl, br] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(bot);

    section(
        frame,
        tl,
        "Braille  h=1  Rail  fade=3",
        Color::Cyan,
        |f, inner| {
            render_pair_in(f, inner, tick, 1, Color::Cyan, |t, s| {
                BarSpinner::new(t)
                    .height(1)
                    .arc_color(Color::Cyan)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        },
    );
    section(
        frame,
        tr,
        "Braille  h=2  Rail  fade=3",
        Color::LightBlue,
        |f, inner| {
            render_pair_in(f, inner, tick, 2, Color::LightBlue, |t, s| {
                BarSpinner::new(t)
                    .height(2)
                    .arc_color(Color::LightBlue)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        },
    );
    section(
        frame,
        bl,
        "Braille  h=1  Full track  fade=0  sharp",
        Color::White,
        |f, inner| {
            render_pair_in(f, inner, tick, 1, Color::White, |t, s| {
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
    section(
        frame,
        br,
        "Braille  h=1  Empty track",
        Color::LightMagenta,
        |f, inner| {
            render_pair_in(f, inner, tick, 1, Color::LightMagenta, |t, s| {
                BarSpinner::new(t)
                    .height(1)
                    .arc_color(Color::LightMagenta)
                    .dim_color(Color::Black)
                    .track(BarTrack::Empty)
                    .spin(s)
            });
        },
    );
}

fn page_symbols(frame: &mut Frame, area: Rect, tick: u64) {
    let [top, bot] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [c0, c1, c2] = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .areas(top);
    let [c3, c4, c5] = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .areas(bot);

    let styles: &[(Rect, BarStyle, &str, Color)] = &[
        (c0, BarStyle::Block, "Block    █  ░", Color::LightGreen),
        (c1, BarStyle::Shade, "Shade    ▓  ░", Color::LightCyan),
        (c2, BarStyle::Dot, "Dot      ●  ·", Color::Yellow),
        (c3, BarStyle::Diamond, "Diamond  ◆  ◇", Color::Magenta),
        (c4, BarStyle::Square, "Square   ■  □", Color::LightRed),
        (c5, BarStyle::Braille, "Braille  ⣿  ⣀", Color::Cyan),
    ];
    for &(cell, style, title, color) in styles {
        section(frame, cell, title, color, |f, inner| {
            render_pair_in(f, inner, tick, 1, color, move |t, s| {
                BarSpinner::new(t)
                    .bar_style(style)
                    .arc_color(color)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        });
    }
}

fn labeled_pair<'a, F>(
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
    let [lbl_row, content, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(height as u16 * 2),
        Constraint::Min(0),
    ])
    .areas(area);
    frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), lbl_row);
    render_pair_in(frame, content, tick, height, color, make);
}

fn page_knobs(frame: &mut Frame, area: Rect, tick: u64) {
    let [top, bot] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
    let [tl, tr] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(top);
    let [bl, br] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(bot);

    section(
        frame,
        tl,
        "arc_width  narrow (5)  vs  wide (20)",
        Color::LightRed,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            labeled_pair(f, ha, tick, "arc_width = 5", 1, Color::LightRed, |t, s| {
                BarSpinner::new(t)
                    .arc_width(5)
                    .arc_color(Color::LightRed)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
            labeled_pair(f, hb, tick, "arc_width = 20", 1, Color::LightRed, |t, s| {
                BarSpinner::new(t)
                    .arc_width(20)
                    .arc_color(Color::LightRed)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        },
    );
    section(
        frame,
        tr,
        "track  Rail  vs  Empty",
        Color::LightYellow,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            labeled_pair(
                f,
                ha,
                tick,
                "Rail  (subtle baseline)",
                1,
                Color::LightYellow,
                |t, s| {
                    BarSpinner::new(t)
                        .track(BarTrack::Rail)
                        .arc_color(Color::LightYellow)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
            labeled_pair(
                f,
                hb,
                tick,
                "Empty  (arc floats)",
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
    section(
        frame,
        bl,
        "fade_width  0 (sharp)  vs  3 (soft)",
        Color::LightMagenta,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            labeled_pair(
                f,
                ha,
                tick,
                "fade=0  sharp",
                1,
                Color::LightMagenta,
                |t, s| {
                    BarSpinner::new(t)
                        .fade_width(0)
                        .arc_color(Color::LightMagenta)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
            labeled_pair(
                f,
                hb,
                tick,
                "fade=3  soft",
                1,
                Color::LightMagenta,
                |t, s| {
                    BarSpinner::new(t)
                        .fade_width(3)
                        .arc_color(Color::LightMagenta)
                        .dim_color(Color::DarkGray)
                        .spin(s)
                },
            );
        },
    );
    section(
        frame,
        br,
        "arc_char  0xFF (full)  vs  0x3F (light)",
        Color::LightCyan,
        |f, inner| {
            let [ha, hb] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner);
            labeled_pair(f, ha, tick, "0xFF full", 1, Color::LightCyan, |t, s| {
                BarSpinner::new(t)
                    .arc_char(0xFF)
                    .arc_color(Color::LightCyan)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
            labeled_pair(f, hb, tick, "0x3F light", 1, Color::LightCyan, |t, s| {
                BarSpinner::new(t)
                    .arc_char(0x3F)
                    .arc_color(Color::LightCyan)
                    .dim_color(Color::DarkGray)
                    .spin(s)
            });
        },
    );
}
