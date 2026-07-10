//! # Embedding Spinners in a Table
//!
//! Demonstrates the headline feature of `tui-spinner`: because every spinner
//! implements `Into<Text>`, it drops straight into any widget that accepts
//! text content — here a Ratatui [`Table`]'s [`Cell`]s.
//!
//! Each row shows a running "task" with a live spinner in its status cell.
//! Two techniques are shown:
//!
//! - **`Cell::from(&spinner)`** — the idiomatic one-liner (uses `Into<Text>`).
//! - **`to_lines()`** — when the spinner rows are combined with other text in
//!   the same cell (see the "Building" row).
//!
//! `BarSpinner` has no intrinsic size, so it uses `to_text(width, height)`.
//!
//! **Controls:** `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example table_embed`

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Cell, Paragraph, Row, Table},
    DefaultTerminal, Frame,
};
use std::time::{Duration, Instant};
use tui_spinner::{
    BarSpinner, CircleSpinner, Direction, FluxSpinner, LinearSpinner, Spin, SquareSpinner,
};

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
        let now = Instant::now();
        let steps = (now.duration_since(app.last_tick).as_millis() / 80).max(1) as u64;
        app.last_tick = now;
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

    render_header(frame, header);
    render_table(frame, body, app.tick);
    render_footer(frame, footer);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled("Spinners live inside ", dim()),
        Span::styled("Table", Style::default().fg(Color::Cyan)),
        Span::styled(" cells via ", dim()),
        Span::styled("Into<Text>", Style::default().fg(Color::LightGreen)),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .title(" tui-spinner · table embedding ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(dim()),
        ),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" / ", dim()),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  Quit", dim()),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(dim()),
        ),
        area,
    );
}

// ── The table ─────────────────────────────────────────────────────────────────

fn render_table(frame: &mut Frame, area: Rect, tick: u64) {
    let header = Row::new([
        Cell::from("Task"),
        Cell::from("Spinner"),
        Cell::from("How it's embedded"),
    ])
    .style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
    .height(1);

    let rows = vec![
        // FluxSpinner — the idiomatic Cell::from(&spinner).
        Row::new([
            Cell::from("Downloading"),
            Cell::from(&FluxSpinner::new(tick).width(8).color(Color::Cyan)),
            Cell::from("Cell::from(&FluxSpinner)"),
        ])
        .height(1),
        // CircleSpinner — by value into a Cell.
        Row::new([
            Cell::from("Indexing"),
            Cell::from(CircleSpinner::new(tick).radius(1).arc_color(Color::Magenta)),
            Cell::from("Cell::from(CircleSpinner)"),
        ])
        .height(3),
        // SquareSpinner — via .into().
        Row::new([
            Cell::from("Compiling"),
            {
                let cell: Cell = SquareSpinner::new(tick)
                    .size(2)
                    .arc_color(Color::Yellow)
                    .into();
                cell
            },
            Cell::from("SquareSpinner.into()"),
        ])
        .height(3),
        // LinearSpinner — horizontal bar embedded by reference.
        Row::new([
            Cell::from("Uploading"),
            Cell::from(
                &LinearSpinner::new(tick)
                    .direction(Direction::Horizontal)
                    .active_color(Color::LightGreen),
            ),
            Cell::from("Cell::from(&LinearSpinner)"),
        ])
        .height(1),
        // BarSpinner — sized, combined with a label using to_lines().
        Row::new([
            Cell::from("Building"),
            {
                let bar = BarSpinner::new(tick).arc_color(Color::LightRed);
                let mut lines: Vec<Line> = vec![Line::from(Span::styled("step 3/5", dim()))];
                lines.extend(bar.to_lines(12, 1));
                Cell::from(lines)
            },
            Cell::from("label + bar.to_lines(w, h)"),
        ])
        .height(2),
        // FluxSpinner counter-clockwise wave — by reference.
        Row::new([
            Cell::from("Syncing"),
            Cell::from(
                &FluxSpinner::new(tick)
                    .width(8)
                    .spin(Spin::CounterClockwise)
                    .color(Color::LightBlue),
            ),
            Cell::from("Cell::from(&FluxSpinner)"),
        ])
        .height(1),
    ];

    let table = Table::new(
        rows,
        [
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .column_spacing(2)
    .block(
        Block::bordered()
            .title(" active tasks ")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(dim()),
    );

    frame.render_widget(table, area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
}
