//! Square braille-arc spinner.
//!
//! A comet-like arc of braille dots travels around the perimeter of a square.
//! The implementation is an exact port of a proven Go algorithm.
//!
//! # Examples
//!
//! ```no_run
//! use ratatui::style::Color;
//! use tui_spinner::{SquareSpinner, Spin, Centre};
//!
//! // Filled center, clockwise
//! let spinner = SquareSpinner::new(42)
//!     .size(3)
//!     .arc_color(Color::Cyan)
//!     .dim_color(Color::DarkGray)
//!     .centre(Centre::Filled)
//!     .spin(Spin::Clockwise);
//!
//! // Empty center, counter-clockwise
//! let hollow = SquareSpinner::new(42)
//!     .size(2)
//!     .arc_color(Color::Green)
//!     .centre(Centre::Empty)
//!     .spin(Spin::CounterClockwise);
//! ```

use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Widget};

use crate::rect_spinner::Spin;

// Re-export Centre so callers can use `tui_spinner::Centre`.
pub use crate::rect_spinner::Centre;

// ── Braille constants ─────────────────────────────────────────────────────────

const BRAILLE_BASE: u32 = 0x2800;

/// Bit index within a braille byte, indexed by `[row % 4][col % 2]`.
const BRAILLE_MAP: [[u8; 2]; 4] = [
    [0, 3], // row 0
    [1, 4], // row 1
    [2, 5], // row 2
    [6, 7], // row 3
];

// ── Internal types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coord {
    row: isize,
    col: isize,
}

impl Coord {
    fn new(row: isize, col: isize) -> Self {
        Self { row, col }
    }
}

struct Grid {
    cells: Vec<Vec<bool>>,
    offset: isize,
}

impl Grid {
    #[allow(clippy::cast_sign_loss)]
    fn set(&mut self, row: isize, col: isize, value: bool) {
        let r = (row + self.offset) as usize;
        let c = col as usize;
        if r < self.cells.len() && c < self.cells[0].len() {
            self.cells[r][c] = value;
        }
    }

    fn fill(&mut self, start: Coord, end: Coord) {
        let x: isize = if end.col < start.col { -1 } else { 1 };
        let y: isize = if end.row < start.row { -1 } else { 1 };

        let mut row = start.row;
        let mut col = start.col;
        self.set(row, col, true);

        while row != end.row {
            row += y;
            self.set(row, col, true);
        }
        while col != end.col {
            col += x;
            self.set(row, col, true);
        }
    }
}

// ── Geometry helpers ──────────────────────────────────────────────────────────

fn calc_dimension(size: usize) -> usize {
    8 + 5 * size.saturating_sub(2)
}

fn vertical_offset(size: usize) -> isize {
    if size == 2 {
        2
    } else {
        0
    }
}

// ── Centre ────────────────────────────────────────────────────────────────────

fn make_centre(size: isize, width: isize) -> (Vec<Coord>, Coord, Coord) {
    let mid = width / 2;
    let off = size / 2;
    let start = Coord::new(mid - off, mid - off);

    let mut cells = Vec::new();
    for i in 0..size {
        for j in 0..size {
            cells.push(Coord::new(start.row + i, start.col + j));
        }
    }
    let end = Coord::new(start.row + size - 1, start.col + size - 1);
    (cells, start, end)
}

// ── Rotation maps ─────────────────────────────────────────────────────────────

fn make_head_map(width: isize, height: isize, size: isize) -> HashMap<Coord, Coord> {
    let mut m = HashMap::new();
    let end_col = width - 1;
    let end_row = height - 1;

    for n in 0..size {
        m.insert(Coord::new(n, end_col), Coord::new(size, end_col - n));
    }
    for n in 0..size {
        m.insert(
            Coord::new(end_row, end_col - n),
            Coord::new(end_row - n, end_col - size),
        );
    }
    for n in 0..size {
        m.insert(Coord::new(end_row - n, 0), Coord::new(end_col - size, n));
    }
    for n in 0..size {
        m.insert(Coord::new(0, n), Coord::new(n, size));
    }
    m
}

fn make_tail_map(width: isize, height: isize, size: isize) -> HashMap<Coord, Coord> {
    let mut m = HashMap::new();
    let end_col = width - 1;
    let end_row = height - 1;

    for n in 0..size {
        m.insert(Coord::new(size, n), Coord::new(n, 0));
    }
    for n in 0..size {
        m.insert(Coord::new(n, end_col - size), Coord::new(0, end_col - n));
    }
    for n in 0..size {
        m.insert(
            Coord::new(end_row - size, end_col - n),
            Coord::new(end_row - n, end_col),
        );
    }
    for n in 0..size {
        m.insert(Coord::new(end_row - n, size), Coord::new(end_row, n));
    }
    m
}

// ── Step logic ────────────────────────────────────────────────────────────────

fn rotate_nodes(nodes: &[Coord], rotation: &HashMap<Coord, Coord>) -> Option<Vec<Coord>> {
    let mut transform = Vec::new();
    for pos in nodes {
        match rotation.get(pos) {
            Some(&next) => transform.push(next),
            None => return None,
        }
    }
    Some(transform)
}

fn x_dir(nodes: &[Coord]) -> isize {
    for pos in nodes {
        if pos.row == 0 {
            return 1;
        }
    }
    -1
}

fn y_dir(nodes: &[Coord]) -> isize {
    for pos in nodes {
        if pos.col == 0 {
            return -1;
        }
    }
    1
}

fn traversing_x(nodes: &[Coord]) -> bool {
    let first_col = nodes[0].col;
    nodes.iter().skip(1).all(|n| n.col == first_col)
}

fn traversing_y(nodes: &[Coord]) -> bool {
    let first_row = nodes[0].row;
    nodes.iter().skip(1).all(|n| n.row == first_row)
}

fn step(nodes: &mut Vec<Coord>, rotate: &HashMap<Coord, Coord>) {
    if let Some(next) = rotate_nodes(nodes, rotate) {
        *nodes = next;
        return;
    }
    if traversing_x(nodes) {
        let dir = x_dir(nodes);
        for n in nodes.iter_mut() {
            n.col += dir;
        }
    }
    if traversing_y(nodes) {
        let dir = y_dir(nodes);
        for n in nodes.iter_mut() {
            n.row += dir;
        }
    }
}

// ── Centre bounds helper ──────────────────────────────────────────────────────

fn should_switch(bounds: &[(usize, usize); 2], row: usize, col: usize) -> bool {
    if row >= bounds[0].0 && row <= bounds[1].0 {
        return col == bounds[0].1 || col == bounds[1].1;
    }
    false
}

// ── Engine ────────────────────────────────────────────────────────────────────

struct SquareEngine {
    grid: Grid,
    head: Vec<Coord>,
    tail: Vec<Coord>,
    head_map: HashMap<Coord, Coord>,
    tail_map: HashMap<Coord, Coord>,
    centre_bounds: [(usize, usize); 2],
    has_centre: bool,
}

impl SquareEngine {
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn build(size: usize, centre: Centre) -> Self {
        let size = size.clamp(2, 8);
        let dm = calc_dimension(size);
        let offset = vertical_offset(size);
        let sz = size as isize;
        let dm_i = dm as isize;

        let total_rows = dm as isize + offset;
        let mut grid = Grid {
            cells: vec![vec![false; dm]; total_rows as usize],
            offset,
        };

        let (centre_cells, c_start, c_end) = make_centre(sz, dm_i);

        let centre_bounds = [
            (
                ((c_start.row + offset) / 4) as usize,
                ((c_start.col / 2) - 1) as usize,
            ),
            (
                ((c_end.row + offset) / 4) as usize,
                (c_end.col / 2) as usize,
            ),
        ];

        let rem = (dm % 2) + ((size - 2) / 2);
        let mid = ((dm / 2) + rem) as isize;

        let head: Vec<Coord> = (0..sz).map(|n| Coord::new(n, mid)).collect();
        let tail: Vec<Coord> = (0..sz).map(|n| Coord::new(mid, n)).collect();

        for i in 0..size {
            grid.fill(tail[i], head[i]);
        }

        let has_centre = matches!(centre, Centre::Filled);
        if has_centre {
            for c in &centre_cells {
                grid.set(c.row, c.col, true);
            }
        }

        let width = dm_i;
        let height = dm_i;

        Self {
            grid,
            head,
            tail,
            head_map: make_head_map(width, height, sz),
            tail_map: make_tail_map(width, height, sz),
            centre_bounds,
            has_centre,
        }
    }

    fn walk(&mut self) {
        step(&mut self.head, &self.head_map);

        for pos in &self.head {
            self.grid.set(pos.row, pos.col, true);
        }
        for pos in &self.tail {
            self.grid.set(pos.row, pos.col, false);
        }

        step(&mut self.tail, &self.tail_map);
    }

    fn render_lines(&self, arc_color: Color, dim_color: Color) -> Vec<Line<'static>> {
        let total_rows = self.grid.cells.len();
        let total_cols = self.grid.cells[0].len();

        let char_rows = total_rows.div_ceil(4);
        let char_cols = total_cols.div_ceil(2);

        let mut screen = vec![vec![0u8; char_cols]; char_rows];

        for (row, row_cells) in self.grid.cells.iter().enumerate() {
            for (col, &on) in row_cells.iter().enumerate() {
                if !on {
                    continue;
                }
                let i = row / 4;
                let j = col / 2;
                let bit = BRAILLE_MAP[row % 4][col % 2];
                screen[i][j] |= 1 << bit;
            }
        }

        let mut lines = Vec::with_capacity(char_rows);
        let mut active = arc_color;

        for (i, row) in screen.iter().enumerate() {
            let mut spans = Vec::with_capacity(char_cols);
            for (j, &b) in row.iter().enumerate() {
                let ch = char::from_u32(BRAILLE_BASE + u32::from(b)).unwrap_or('\u{2800}');
                spans.push(Span::styled(ch.to_string(), Style::default().fg(active)));

                if self.has_centre && should_switch(&self.centre_bounds, i, j) {
                    active = if active == arc_color {
                        dim_color
                    } else {
                        arc_color
                    };
                }
            }
            lines.push(Line::from(spans));
        }

        lines
    }
}

// ── Public widget ─────────────────────────────────────────────────────────────

/// A rotating square braille-arc spinner.
///
/// A comet-like arc of braille dots travels around the perimeter of a square.
/// Supports filled or empty center modes and clockwise/counter-clockwise
/// directions.
///
/// # Quick start
///
/// ```no_run
/// use ratatui::style::Color;
/// use tui_spinner::{Centre, SquareSpinner, Spin};
///
/// let spinner = SquareSpinner::new(42)
///     .size(3)
///     .arc_color(Color::Cyan)
///     .dim_color(Color::DarkGray)
///     .centre(Centre::Filled);
/// ```
#[derive(Debug, Clone)]
pub struct SquareSpinner<'a> {
    tick: u64,
    size: usize,
    ticks_per_step: u64,
    spin: Spin,
    centre: Centre,
    /// Colour of the rotating bright arc.
    arc_color: Color,
    /// Colour of the filled centre (when `Centre::Filled`).
    dim_color: Color,
    block: Option<Block<'a>>,
    style: Style,
    alignment: Alignment,
}

impl<'a> SquareSpinner<'a> {
    /// Creates a new [`SquareSpinner`] with defaults: size 2, clockwise spin,
    /// white arc, dark-gray dim, filled centre, 1 tick per step.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::SquareSpinner;
    ///
    /// let spinner = SquareSpinner::new(42);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            size: 2,
            ticks_per_step: 1,
            spin: Spin::Clockwise,
            centre: Centre::Filled,
            arc_color: Color::White,
            dim_color: Color::DarkGray,
            block: None,
            style: Style::default(),
            alignment: Alignment::Left,
        }
    }

    /// Sets the arc thickness / square size (default: 2, range: 2–8).
    ///
    /// Larger values produce a bigger square with a thicker arc.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::SquareSpinner;
    ///
    /// let large = SquareSpinner::new(0).size(4);
    /// ```
    #[must_use]
    pub fn size(mut self, size: usize) -> Self {
        self.size = size.clamp(2, 8);
        self
    }

    /// Sets the spin direction (default: [`Spin::Clockwise`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{SquareSpinner, Spin};
    ///
    /// let ccw = SquareSpinner::new(0).spin(Spin::CounterClockwise);
    /// ```
    #[must_use]
    pub const fn spin(mut self, spin: Spin) -> Self {
        self.spin = spin;
        self
    }

    /// Controls whether the centre is filled or empty (default: [`Centre::Filled`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{Centre, SquareSpinner};
    ///
    /// let hollow = SquareSpinner::new(0).centre(Centre::Empty);
    /// ```
    #[must_use]
    pub const fn centre(mut self, centre: Centre) -> Self {
        self.centre = centre;
        self
    }

    /// Sets the colour of the rotating bright arc (default: [`Color::White`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::SquareSpinner;
    ///
    /// let spinner = SquareSpinner::new(0).arc_color(Color::Cyan);
    /// ```
    #[must_use]
    pub const fn arc_color(mut self, color: Color) -> Self {
        self.arc_color = color;
        self
    }

    /// Sets the colour of the filled centre region (default: [`Color::DarkGray`]).
    ///
    /// Only visible when [`Centre::Filled`] is active.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::SquareSpinner;
    ///
    /// let spinner = SquareSpinner::new(0).dim_color(Color::DarkGray);
    /// ```
    #[must_use]
    pub const fn dim_color(mut self, color: Color) -> Self {
        self.dim_color = color;
        self
    }

    /// Sets how many ticks each arc position is held (default: 1, higher = slower).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::SquareSpinner;
    ///
    /// let slow = SquareSpinner::new(0).ticks_per_step(3);
    /// ```
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.ticks_per_step = n.max(1);
        self
    }

    /// Wraps the spinner in a [`Block`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::widgets::Block;
    /// use tui_spinner::SquareSpinner;
    ///
    /// let spinner = SquareSpinner::new(0).block(Block::bordered().title("Loading…"));
    /// ```
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the base style applied to the widget area.
    #[must_use]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the horizontal alignment of the rendered output (default: left).
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Returns the exact rendered size in terminal characters `(cols, rows)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::SquareSpinner;
    ///
    /// let (cols, rows) = SquareSpinner::new(0).size(2).char_size();
    /// assert_eq!(cols, 4);
    /// assert_eq!(rows, 3);
    /// ```
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn char_size(&self) -> (u16, u16) {
        let dm = calc_dimension(self.size);
        let offset = vertical_offset(self.size) as usize;
        let total_rows = dm + offset;
        let char_cols = dm.div_ceil(2);
        let char_rows = total_rows.div_ceil(4);
        (char_cols as u16, char_rows as u16)
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut engine = SquareEngine::build(self.size, self.centre);

        #[allow(clippy::cast_possible_truncation)]
        let steps = (self.tick / self.ticks_per_step) as usize;
        for _ in 0..steps {
            engine.walk();
        }

        let mut lines = engine.render_lines(self.arc_color, self.dim_color);

        if matches!(self.spin, Spin::CounterClockwise) {
            for line in &mut lines {
                line.spans.reverse();
            }
        }

        lines
    }
}

impl_styled_for!(SquareSpinner<'_>);

impl_widget_via_ref!(SquareSpinner<'_>);

impl Widget for &SquareSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_spinner_body!(self, area, buf, self.build_lines());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_all_sizes() {
        for size in 2..=6 {
            for centre in [Centre::Filled, Centre::Empty] {
                let e = SquareEngine::build(size, centre);
                assert!(!e.head.is_empty());
                assert!(!e.tail.is_empty());
            }
        }
    }

    #[test]
    fn walk_does_not_panic() {
        for size in 2..=4 {
            let mut e = SquareEngine::build(size, Centre::Filled);
            let dm = calc_dimension(size);
            for _ in 0..dm * 8 {
                e.walk();
            }
        }
    }

    #[test]
    fn filled_vs_empty_differ() {
        let filled = SquareEngine::build(2, Centre::Filled);
        let empty = SquareEngine::build(2, Centre::Empty);
        let lf = filled.render_lines(Color::Cyan, Color::DarkGray);
        let le = empty.render_lines(Color::Cyan, Color::DarkGray);
        assert_ne!(lf, le);
    }

    #[test]
    fn widget_renders_without_panic() {
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        Widget::render(&SquareSpinner::new(0), area, &mut buf);
    }

    #[test]
    fn cw_and_ccw_differ() {
        let area = Rect::new(0, 0, 20, 10);
        let mut b1 = Buffer::empty(area);
        let mut b2 = Buffer::empty(area);
        Widget::render(&SquareSpinner::new(0).spin(Spin::Clockwise), area, &mut b1);
        Widget::render(
            &SquareSpinner::new(0).spin(Spin::CounterClockwise),
            area,
            &mut b2,
        );
        assert_ne!(b1, b2);
    }

    #[test]
    fn different_ticks_differ() {
        let area = Rect::new(0, 0, 20, 10);
        let mut b0 = Buffer::empty(area);
        let mut b5 = Buffer::empty(area);
        Widget::render(&SquareSpinner::new(0), area, &mut b0);
        Widget::render(&SquareSpinner::new(5), area, &mut b5);
        assert_ne!(b0, b5);
    }

    #[test]
    fn char_size_is_correct() {
        let (cols, rows) = SquareSpinner::new(0).size(2).char_size();
        assert_eq!(cols, 4);
        assert_eq!(rows, 3);

        let (cols3, rows3) = SquareSpinner::new(0).size(3).char_size();
        assert!(cols3 > cols);
        assert!(rows3 > rows);
    }

    #[test]
    fn zero_area_no_panic() {
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        Widget::render(&SquareSpinner::new(0), area, &mut buf);
    }

    #[test]
    fn builder_chain() {
        let s = SquareSpinner::new(10)
            .size(4)
            .spin(Spin::CounterClockwise)
            .centre(Centre::Empty)
            .arc_color(Color::Cyan)
            .dim_color(Color::DarkGray)
            .ticks_per_step(3)
            .alignment(Alignment::Center);

        assert_eq!(s.size, 4);
        assert_eq!(s.spin, Spin::CounterClockwise);
        assert_eq!(s.centre, Centre::Empty);
        assert_eq!(s.arc_color, Color::Cyan);
        assert_eq!(s.dim_color, Color::DarkGray);
        assert_eq!(s.ticks_per_step, 3);
    }
}
