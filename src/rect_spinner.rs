//! Square braille-arc spinner — exact port of the Go implementation.

use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Widget};

const BRAILLE_BASE: u32 = 0x2800;

/// Braille bit index, indexed by `[row % 4][col % 2]`.
const BRAILLE_MAP: [[u8; 2]; 4] = [
    [0, 3], // row 0
    [1, 4], // row 1
    [2, 5], // row 2
    [6, 7], // row 3
];

// ── Public enums ──────────────────────────────────────────────────────────────

/// Rotation direction of the arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Spin {
    /// Arc travels clockwise around the perimeter (default).
    #[default]
    Clockwise,
    /// Arc travels counter-clockwise around the perimeter.
    CounterClockwise,
}

/// Whether the centre of the spinner is filled or empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Centre {
    /// The interior is filled with a solid block.
    #[default]
    Filled,
    /// The interior is left empty — only the moving arc is visible.
    Empty,
}

/// Shape of the spinner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectShape {
    /// A square spinner with arc thickness parameter (size 2–8).
    Square(usize),
}

impl Default for RectShape {
    fn default() -> Self {
        Self::Square(2)
    }
}

// ── Internal types — exact port of Go structs ─────────────────────────────────

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
    /// `set` applies the offset, matching `grid.cells[row+grid.offset][col]`.
    #[allow(clippy::cast_sign_loss)]
    fn set(&mut self, row: isize, col: isize, value: bool) {
        let r = (row + self.offset) as usize;
        let c = col as usize;
        if r < self.cells.len() && c < self.cells[0].len() {
            self.cells[r][c] = value;
        }
    }

    /// `fill` walks an L-shaped path from `start` to `end`, setting every cell
    /// to `true`.  Exact port of Go `Grid.fill`.
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

/// `calc_dimension` — Go: `low := 8; x := 5 * (size - 2); return low + x`
fn calc_dimension(size: usize) -> usize {
    8 + 5 * size.saturating_sub(2)
}

/// `vertical_offset` — Go: `if size == 2 { return 2 } return 0`
fn vertical_offset(size: usize) -> isize {
    if size == 2 {
        2
    } else {
        0
    }
}

// ── Centre ────────────────────────────────────────────────────────────────────

/// Go: `make_centre(size, width)` returns `(map, start, end)`.
/// Here we return the centre cell set and the two bounding `Coord`s.
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

/// Exact port of Go `make_head_map`.
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

/// Exact port of Go `make_tail_map`.
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

// ── Step logic — exact port ───────────────────────────────────────────────────

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

/// Exact port of Go `step`.  Note: Go uses **two separate `if`s**, not
/// `if / else if`.  We replicate that.
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

/// Go: `should_switch(bounds [2]Coord, row, col)`
fn should_switch(bounds: &[(usize, usize); 2], row: usize, col: usize) -> bool {
    if row >= bounds[0].0 && row <= bounds[1].0 {
        return col == bounds[0].1 || col == bounds[1].1;
    }
    false
}

// ── Engine — combines Grid + head/tail + rotation maps ────────────────────────

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
    /// Exact port of Go `makeSpinner`.
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn build(size: usize, centre: Centre, _spin: Spin) -> Self {
        let size = size.clamp(2, 8);
        let dm = calc_dimension(size);
        let offset = vertical_offset(size);
        let sz = size as isize;
        let dm_i = dm as isize;

        // Go: cells = make([][]bool, dm+offset); cells[i] = make([]bool, dm)
        let total_rows = dm as isize + offset;
        let mut grid = Grid {
            cells: vec![vec![false; dm]; total_rows as usize],
            offset,
        };

        // Go: centre_map, start, end := make_centre(size, dm)
        let (centre_cells, c_start, c_end) = make_centre(sz, dm_i);

        // Go: bounds := [2]Coord{
        //   {row: (start.row + grid.offset) / 4, col: (start.col / 2) - 1},
        //   {row: (end.row + grid.offset) / 4,   col: end.col / 2},
        // }
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

        // Go: head, tail := key_nodes(dm, size)
        let rem = (dm % 2) + ((size - 2) / 2);
        let mid = ((dm / 2) + rem) as isize;

        let head: Vec<Coord> = (0..sz).map(|n| Coord::new(n, mid)).collect();
        let tail: Vec<Coord> = (0..sz).map(|n| Coord::new(mid, n)).collect();

        // Go: for i := range head { grid.fill(tail[i], head[i]) }
        for i in 0..size {
            grid.fill(tail[i], head[i]);
        }

        // Go: for c := range centre_map { grid.set(c.row, c.col, true) }
        let has_centre = matches!(centre, Centre::Filled);
        if has_centre {
            for c in &centre_cells {
                grid.set(c.row, c.col, true);
            }
        }

        // Go: width, height := len(cells)-offset, len(cells[0])
        // len(cells) = dm+offset, so width = dm.   len(cells[0]) = dm, so height = dm.
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

    /// Exact port of Go `walk`.
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

    /// Exact port of Go `render_frame`.
    fn render_frame(&self, outer_color: Color, inner_color: Color) -> Vec<Line<'static>> {
        let total_rows = self.grid.cells.len();
        let total_cols = self.grid.cells[0].len();

        // Go: height := (len(sp.grid.cells) + 3) / 4
        //     width  := (len(sp.grid.cells[0]) + 1) / 2
        let char_rows = total_rows.div_ceil(4);
        let char_cols = total_cols.div_ceil(2);

        let mut screen = vec![vec![0u8; char_cols]; char_rows];

        // Pack dots into braille bytes
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

        // Render into Lines
        let mut lines = Vec::with_capacity(char_rows);
        let mut active = outer_color;

        for (i, row) in screen.iter().enumerate() {
            let mut spans = Vec::with_capacity(char_cols);
            for (j, &b) in row.iter().enumerate() {
                let ch = char::from_u32(BRAILLE_BASE + u32::from(b)).unwrap_or('\u{2800}');
                spans.push(Span::styled(ch.to_string(), Style::default().fg(active)));

                if self.has_centre && should_switch(&self.centre_bounds, i, j) {
                    active = if active == outer_color {
                        inner_color
                    } else {
                        outer_color
                    };
                }
            }
            lines.push(Line::from(spans));
        }

        lines
    }
}

// ── Public widget ─────────────────────────────────────────────────────────────

/// A simple square braille-arc spinner with filled or empty center.
#[derive(Debug, Clone)]
pub struct RectSpinner<'a> {
    tick: u64,
    shape: RectShape,
    spin: Spin,
    ticks_per_step: u64,
    outer_color: Color,
    inner_color: Color,
    centre: Centre,
    block: Option<Block<'a>>,
    style: Style,
    alignment: Alignment,
}

impl<'a> RectSpinner<'a> {
    /// Creates a new [`RectSpinner`] at the given tick.
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            shape: RectShape::default(),
            spin: Spin::default(),
            ticks_per_step: 1,
            outer_color: Color::Cyan,
            inner_color: Color::DarkGray,
            centre: Centre::default(),
            block: None,
            style: Style::default(),
            alignment: Alignment::Left,
        }
    }

    /// Sets the shape and size.
    #[must_use]
    pub const fn shape(mut self, shape: RectShape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the rotation direction.
    #[must_use]
    pub const fn spin(mut self, spin: Spin) -> Self {
        self.spin = spin;
        self
    }

    /// Sets the outer arc color.
    #[must_use]
    pub const fn outer_color(mut self, color: Color) -> Self {
        self.outer_color = color;
        self
    }

    /// Sets the inner centre color.
    #[must_use]
    pub const fn inner_color(mut self, color: Color) -> Self {
        self.inner_color = color;
        self
    }

    /// Sets the centre fill mode.
    #[must_use]
    pub const fn centre(mut self, centre: Centre) -> Self {
        self.centre = centre;
        self
    }

    /// Sets ticks per step (higher = slower).
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.ticks_per_step = n.max(1);
        self
    }

    /// Wraps the spinner in a [`Block`].
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the base style.
    #[must_use]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Sets horizontal alignment.
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    fn render_lines(&self) -> Vec<Line<'static>> {
        let mut engine = match self.shape {
            RectShape::Square(size) => SquareEngine::build(size, self.centre, self.spin),
        };

        let steps = self.tick / self.ticks_per_step;
        for _ in 0..steps {
            engine.walk();
        }

        let mut lines = engine.render_frame(self.outer_color, self.inner_color);

        if matches!(self.spin, Spin::CounterClockwise) {
            for line in &mut lines {
                line.spans.reverse();
            }
        }

        lines
    }
}

impl_styled_for!(RectSpinner<'_>);

impl_to_text!(RectSpinner<'_>, render_lines);

impl_widget_via_ref!(RectSpinner<'_>);

impl Widget for &RectSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_spinner_body!(self, area, buf, self.render_lines());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_engine_builds() {
        for size in 2..=6 {
            for centre in [Centre::Filled, Centre::Empty] {
                let e = SquareEngine::build(size, centre, Spin::Clockwise);
                assert!(!e.head.is_empty());
                assert!(!e.tail.is_empty());
            }
        }
    }

    #[test]
    fn square_engine_walk_does_not_panic() {
        for size in 2..=4 {
            let mut e = SquareEngine::build(size, Centre::Filled, Spin::Clockwise);
            let dm = calc_dimension(size);
            for _ in 0..dm * 8 {
                e.walk();
            }
        }
    }

    #[test]
    fn filled_vs_empty_differ() {
        let filled = SquareEngine::build(2, Centre::Filled, Spin::Clockwise);
        let empty = SquareEngine::build(2, Centre::Empty, Spin::Clockwise);

        let lf = filled.render_frame(Color::Cyan, Color::DarkGray);
        let le = empty.render_frame(Color::Cyan, Color::DarkGray);

        assert_ne!(lf, le);
    }

    #[test]
    fn widget_renders() {
        let area = Rect::new(0, 0, 20, 10);
        let mut buf = Buffer::empty(area);
        Widget::render(&RectSpinner::new(0), area, &mut buf);
    }

    #[test]
    fn cw_and_ccw_differ() {
        let area = Rect::new(0, 0, 20, 10);
        let mut b1 = Buffer::empty(area);
        let mut b2 = Buffer::empty(area);

        Widget::render(&RectSpinner::new(0).spin(Spin::Clockwise), area, &mut b1);
        Widget::render(
            &RectSpinner::new(0).spin(Spin::CounterClockwise),
            area,
            &mut b2,
        );

        assert_ne!(b1, b2);
    }

    #[test]
    fn different_ticks_produce_different_output() {
        let area = Rect::new(0, 0, 20, 10);
        let mut b0 = Buffer::empty(area);
        let mut b5 = Buffer::empty(area);
        Widget::render(&RectSpinner::new(0), area, &mut b0);
        Widget::render(&RectSpinner::new(5), area, &mut b5);
        assert_ne!(b0, b5);
    }

    #[test]
    fn to_lines_matches_render_lines() {
        let s = RectSpinner::new(3);
        assert_eq!(s.to_lines(), s.render_lines());
        assert_eq!(s.to_text().lines.len(), s.render_lines().len());
    }
}
