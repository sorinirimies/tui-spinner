//! Rectangular braille-arc spinner.
//!
//! A comet-like arc of braille dots travels around the perimeter of a
//! rectangle.  The geometry is designed so that the rendered output occupies
//! a **square** character-cell grid:
//!
//! - `calc_dimension(size)` → `dot_cols` (number of dot columns).
//! - `dot_rows = 2 * dot_cols` — twice as many dot-rows as dot-columns so
//!   that after braille packing (`÷2` cols, `÷4` rows) the output is
//!   `dot_cols/2` chars wide × `dot_cols/2` chars tall — a square.
//! - `vertical_offset(size)` adds a small top padding for size-2 grids.
//! - Four corner-turn maps encode the clockwise (or counter-clockwise)
//!   rotation; reversing the maps gives the opposite spin direction.
//! - The centre region is filled with a solid block that alternates colour
//!   each time the arc crosses the centre column boundary (`Centre::Filled`),
//!   or left empty (`Centre::Empty`).
//!
//! ## Shapes
//!
//! | [`RectShape`]    | Description                                          |
//! |------------------|------------------------------------------------------|
//! | `Square(size)`   | Square char-cell output; `size` controls arc width   |
//! | `Narrow(height)` | 1-char-wide Zed-style sliding-arc column             |
//!
//! ## Spin direction
//!
//! | [`Spin`]              | Description               |
//! |-----------------------|---------------------------|
//! | `Spin::Clockwise`     | Arc travels clockwise (default) |
//! | `Spin::CounterClockwise` | Arc travels counter-clockwise |
//!
//! ## Centre
//!
//! | [`Centre`]    | Description                                         |
//! |---------------|-----------------------------------------------------|
//! | `Filled`      | Solid interior, colour-switches on each pass        |
//! | `Empty`       | No interior fill — only the moving arc is visible   |
//!
//! ## Styles
//!
//! | [`RectStyle`] | Arc cell             | Empty cell          |
//! |---------------|----------------------|---------------------|
//! | `Arc`         | braille dot-pattern  | `⠀` blank braille   |
//! | `Dense`       | `⣿` full braille    | `⠀` blank braille   |
//! | `Shade`       | `█` full block       | `░` light shade     |
//! | `Outline`     | `◉` fisheye          | `○` open circle     |
//! | `Dot`         | `•` bullet           | `·` middle dot      |
//! | `Star`        | `★` filled star      | `☆` open star       |
//! | `Diamond`     | `◆` filled diamond   | `◇` open diamond    |
//! | `Cross`       | `╋` heavy plus       | `┼` light plus      |
//! | `Fade`        | `█`/`▓`/`▒` density  | `░` light shade     |
//! | `Pixel`       | `▪` small square     | `▫` small open sq   |

use std::collections::HashMap;

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style, Styled};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

// ── Constants ─────────────────────────────────────────────────────────────────

const BRAILLE_BASE: u32 = 0x2800;

/// Braille bit index, indexed by `(dot_row % 4, dot_col % 2)`.
const BRAILLE_MAP: [[u8; 2]; 4] = [
    [0, 3], // row 0: col 0 → bit 0,  col 1 → bit 3
    [1, 4], // row 1: col 0 → bit 1,  col 1 → bit 4
    [2, 5], // row 2: col 0 → bit 2,  col 1 → bit 5
    [6, 7], // row 3: col 0 → bit 6,  col 1 → bit 7
];

// ── Public enums ──────────────────────────────────────────────────────────────

/// Rotation direction of the arc.
///
/// # Examples
///
/// ```
/// use tui_spinner::{RectSpinner, Spin};
///
/// let cw  = RectSpinner::new(0).spin(Spin::Clockwise);
/// let ccw = RectSpinner::new(0).spin(Spin::CounterClockwise);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Spin {
    /// Arc travels clockwise around the perimeter (default).
    #[default]
    Clockwise,
    /// Arc travels counter-clockwise around the perimeter.
    CounterClockwise,
}

/// Whether the centre of the spinner is filled with a solid block or left empty.
///
/// # Examples
///
/// ```
/// use tui_spinner::{Centre, RectSpinner};
///
/// let filled = RectSpinner::new(0).centre(Centre::Filled);
/// let hollow = RectSpinner::new(0).centre(Centre::Empty);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Centre {
    /// The interior of the arc path is filled with a solid block that
    /// alternates between `outer_color` and `inner_color` each time the arc
    /// crosses the centre column boundary.
    #[default]
    Filled,
    /// No centre fill.  Only the travelling arc itself is drawn; the interior
    /// is left blank.
    Empty,
}

/// The overall shape of the [`RectSpinner`].
///
/// # Examples
///
/// ```
/// use tui_spinner::{RectShape, RectSpinner};
///
/// let sq     = RectSpinner::new(0).shape(RectShape::Square(2));
/// let narrow = RectSpinner::new(0).shape(RectShape::Narrow(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectShape {
    /// A square spinner whose arc travels a square character-cell path.
    ///
    /// The argument is the arc-thickness / size parameter (minimum 2).
    /// `dot_cols = 8 + 5*(size-2)`, `dot_rows = 2*dot_cols`, giving a square
    /// `dot_cols/2 × dot_cols/2` character-cell output.
    Square(usize),
    /// A 1-character-wide Zed-style sidebar spinner.
    ///
    /// The argument is the height in terminal character rows (minimum 3).
    Narrow(usize),
}

impl Default for RectShape {
    fn default() -> Self {
        Self::Square(2)
    }
}

/// Controls how individual arc cells are rendered.
///
/// Braille-based styles (`Arc`, `Dense`, `Fade`) pack 4 dot-rows and 2
/// dot-columns into each terminal character.  All other styles render exactly
/// one character per collapsed char-cell.
///
/// # Examples
///
/// ```
/// use tui_spinner::{RectSpinner, RectStyle};
///
/// let arc   = RectSpinner::new(0).render_style(RectStyle::Arc);
/// let dense = RectSpinner::new(0).render_style(RectStyle::Dense);
/// let shade = RectSpinner::new(0).render_style(RectStyle::Shade);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RectStyle {
    /// Exact braille dot-pattern (default).
    #[default]
    Arc,
    /// Solid `⣿` arc cells.
    Dense,
    /// `█` / `░` block characters.
    Shade,
    /// `◉` / `○` circle symbols.
    Outline,
    /// `•` / `·` bullet / middle dot.
    Dot,
    /// `★` / `☆` filled / open star.
    Star,
    /// `◆` / `◇` filled / open diamond.
    Diamond,
    /// `╋` / `┼` heavy / light plus.
    Cross,
    /// `█`/`▓`/`▒` by braille bit density.
    Fade,
    /// `▪` / `▫` small filled / open square.
    Pixel,
}

impl RectStyle {
    /// Returns `(arc_char, empty_char)` for this style.
    ///
    /// For braille styles `byte` is the packed braille byte value; for all
    /// other styles only whether `byte != 0` is used.
    pub fn cell_chars(self, byte: u8) -> (char, char) {
        match self {
            Self::Arc => {
                let ch = if byte == 0 {
                    '\u{2800}'
                } else {
                    char::from_u32(BRAILLE_BASE + u32::from(byte)).unwrap_or('\u{2800}')
                };
                (ch, '\u{2800}')
            }
            Self::Dense => (
                char::from_u32(BRAILLE_BASE + 0xFF).unwrap_or('⣿'),
                '\u{2800}',
            ),
            Self::Shade => ('█', '░'),
            Self::Outline => ('◉', '○'),
            Self::Dot => ('•', '·'),
            Self::Star => ('★', '☆'),
            Self::Diamond => ('◆', '◇'),
            Self::Cross => ('╋', '┼'),
            Self::Fade => {
                let bits = byte.count_ones();
                let ch = match bits {
                    0 => '░',
                    1..=3 => '▒',
                    4..=6 => '▓',
                    _ => '█',
                };
                (ch, '░')
            }
            Self::Pixel => ('▪', '▫'),
        }
    }

    /// Returns `true` for styles that use braille characters.
    pub const fn is_braille(self) -> bool {
        matches!(self, Self::Arc | Self::Dense | Self::Fade)
    }
}

// ── Internal geometry helpers ─────────────────────────────────────────────────

/// Number of dot columns for a given `size`.
///
/// `dim = 8 + 5*(size-2)`:  size 2 → 8,  size 3 → 13,  size 4 → 18, …
fn calc_dot_cols(size: usize) -> usize {
    8 + 5 * size.saturating_sub(2)
}

/// Number of dot rows for a given `size`.
///
/// Exactly `2 * dot_cols` so that after braille packing (`÷2` cols, `÷4` rows)
/// the output is a square `dot_cols/2 × dot_cols/2` character-cell grid.
fn calc_dot_rows(size: usize) -> usize {
    2 * calc_dot_cols(size)
}

// ── Coordinate ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coord {
    row: isize,
    col: isize,
}

impl Coord {
    const fn new(row: isize, col: isize) -> Self {
        Self { row, col }
    }
}

// ── Boolean dot-grid ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct Grid {
    /// Row-major cells; total rows = `dot_rows + offset`, cols = `dot_cols`.
    cells: Vec<Vec<bool>>,
    /// Extra blank rows prepended above the active area.
    offset: usize,
    /// Number of dot columns.
    dot_cols: usize,
}

impl Grid {
    fn new(dot_cols: usize, dot_rows: usize, offset: usize) -> Self {
        Self {
            cells: vec![vec![false; dot_cols]; dot_rows + offset],
            offset,
            dot_cols,
        }
    }

    #[inline]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    fn set(&mut self, row: isize, col: isize, value: bool) {
        let r = (row + self.offset as isize) as usize;
        let c = col as usize;
        if r < self.cells.len() && c < self.dot_cols {
            self.cells[r][c] = value;
        }
    }

    /// Fill an axis-aligned path from `start` to `end`.
    fn fill(&mut self, start: Coord, end: Coord) {
        let dy: isize = match end.row.cmp(&start.row) {
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
        };
        let dx: isize = match end.col.cmp(&start.col) {
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
        };
        let mut row = start.row;
        let mut col = start.col;
        self.set(row, col, true);
        while row != end.row {
            row += dy;
            self.set(row, col, true);
        }
        while col != end.col {
            col += dx;
            self.set(row, col, true);
        }
    }
}

// ── Corner-turn rotation maps ─────────────────────────────────────────────────
//
// `make_head_map` / `make_tail_map` encode the four clockwise corner turns.
// Passing `reversed = true` builds the counter-clockwise equivalents by
// swapping each map's source and destination.

fn make_head_map(
    width: isize,
    height: isize,
    size: isize,
    reversed: bool,
) -> HashMap<Coord, Coord> {
    let ec = width - 1;
    let er = height - 1;
    let mut fwd: Vec<(Coord, Coord)> = Vec::new();

    // Top-right corner: heading down → heading left
    for n in 0..size {
        fwd.push((Coord::new(n, ec), Coord::new(size, ec - n)));
    }
    // Bottom-right corner: heading left → heading up
    for n in 0..size {
        fwd.push((Coord::new(er, ec - n), Coord::new(er - n, ec - size)));
    }
    // Bottom-left corner: heading up → heading right
    for n in 0..size {
        fwd.push((Coord::new(er - n, 0), Coord::new(ec - size, n)));
    }
    // Top-left corner: heading right → heading down
    for n in 0..size {
        fwd.push((Coord::new(0, n), Coord::new(n, size)));
    }

    if reversed {
        fwd.into_iter().map(|(a, b)| (b, a)).collect()
    } else {
        fwd.into_iter().collect()
    }
}

fn make_tail_map(
    width: isize,
    height: isize,
    size: isize,
    reversed: bool,
) -> HashMap<Coord, Coord> {
    let ec = width - 1;
    let er = height - 1;
    let mut fwd: Vec<(Coord, Coord)> = Vec::new();

    for n in 0..size {
        fwd.push((Coord::new(size, n), Coord::new(n, 0)));
    }
    for n in 0..size {
        fwd.push((Coord::new(n, ec - size), Coord::new(0, ec - n)));
    }
    for n in 0..size {
        fwd.push((Coord::new(er - size, ec - n), Coord::new(er - n, ec)));
    }
    for n in 0..size {
        fwd.push((Coord::new(er - n, size), Coord::new(er, n)));
    }

    if reversed {
        fwd.into_iter().map(|(a, b)| (b, a)).collect()
    } else {
        fwd.into_iter().collect()
    }
}

// ── Step logic ────────────────────────────────────────────────────────────────

fn rotate_nodes(nodes: &[Coord], map: &HashMap<Coord, Coord>) -> Option<Vec<Coord>> {
    nodes.iter().map(|c| map.get(c).copied()).collect()
}

/// True when all nodes share the same column (arc moving vertically).
fn traversing_x(nodes: &[Coord]) -> bool {
    nodes.windows(2).all(|w| w[0].col == w[1].col)
}

/// True when all nodes share the same row (arc moving horizontally).
fn traversing_y(nodes: &[Coord]) -> bool {
    nodes.windows(2).all(|w| w[0].row == w[1].row)
}

/// Direction of vertical travel for clockwise motion: +1 (down) or -1 (up).
fn x_dir(nodes: &[Coord]) -> isize {
    if nodes.iter().any(|c| c.row == 0) {
        1
    } else {
        -1
    }
}

/// Direction of horizontal travel for clockwise motion: -1 (left) or +1 (right).
fn y_dir(nodes: &[Coord]) -> isize {
    if nodes.iter().any(|c| c.col == 0) {
        -1
    } else {
        1
    }
}

/// Advance a set of nodes one step along the perimeter.
fn step(nodes: &mut Vec<Coord>, map: &HashMap<Coord, Coord>) {
    if let Some(rotated) = rotate_nodes(nodes, map) {
        *nodes = rotated;
        return;
    }
    if traversing_x(nodes) {
        let dir = x_dir(nodes);
        for c in nodes.iter_mut() {
            c.col += dir;
        }
    } else if traversing_y(nodes) {
        let dir = y_dir(nodes);
        for c in nodes.iter_mut() {
            c.row += dir;
        }
    }
}

// ── Centre bounds (colour switching) ─────────────────────────────────────────

/// Bounds of the centre fill in braille character coordinates.
#[derive(Debug, Clone, Copy)]
struct CentreBounds {
    top_left: (usize, usize),
    bottom_right: (usize, usize),
}

impl CentreBounds {
    /// Returns `true` when the arc is at a vertical edge of the centre region,
    /// triggering the colour toggle.
    fn should_switch(self, char_row: usize, char_col: usize) -> bool {
        let (r0, c0) = self.top_left;
        let (r1, c1) = self.bottom_right;
        (char_row >= r0 && char_row <= r1) && (char_col == c0 || char_col == c1)
    }
}

/// Sentinel used when there is no centre fill (`Centre::Empty`).
const NO_CENTRE: CentreBounds = CentreBounds {
    top_left: (usize::MAX, usize::MAX),
    bottom_right: (0, 0),
};

// ── Arc engine ────────────────────────────────────────────────────────────────

/// All mutable state for one frame of the square arc animation.
#[derive(Debug, Clone)]
struct ArcEngine {
    grid: Grid,
    head: Vec<Coord>,
    tail: Vec<Coord>,
    head_map: HashMap<Coord, Coord>,
    tail_map: HashMap<Coord, Coord>,
    centre_bounds: CentreBounds,
}

impl ArcEngine {
    /// Build a square engine.
    ///
    /// The dot grid is `dot_cols × dot_rows` where `dot_rows = 2 * dot_cols`,
    /// producing a square `dot_cols/2 × dot_cols/2` character-cell output.
    ///
    /// For `Spin::CounterClockwise` the grid is built identically but the
    /// `spin` flag is stored so that `render_lines` can mirror each row
    /// horizontally, giving a visually counter-clockwise arc.
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn build_square(size: usize, centre: Centre, _spin: Spin) -> Self {
        let size = size.clamp(2, 8);
        let dot_cols = calc_dot_cols(size);
        let dot_rows = calc_dot_rows(size); // = 2 * dot_cols
        let sz = size as isize;

        // No vertical offset — dot_rows = 2*dot_cols already gives a square
        // char-cell output without any padding.
        let mut grid = Grid::new(dot_cols, dot_rows, 0);

        // ── Centre square ──────────────────────────────────────────────────────
        // To render as a square in character-cell space we need the filled
        // region to be N char-cols × N char-rows.  Since 1 char-col = 2 dot-cols
        // and 1 char-row = 4 dot-rows, the dot dimensions are 2N × 4N.
        // We pick N = size / 2 (minimum 1) so the centre scales with arc size.
        let n_chars = (size / 2).max(1); // centre square side in char-cells
        let centre_half_cols = n_chars * 2; // half-width  in dot columns
        let centre_half_rows = n_chars * 4; // half-height in dot rows

        let col_mid = dot_cols / 2;
        let row_mid = dot_rows / 2;

        let c_start = Coord::new(
            (row_mid as isize) - (centre_half_rows as isize),
            (col_mid as isize) - (centre_half_cols as isize),
        );
        let c_end = Coord::new(
            (row_mid as isize) + (centre_half_rows as isize) - 1,
            (col_mid as isize) + (centre_half_cols as isize) - 1,
        );

        let centre_bounds = if matches!(centre, Centre::Filled) {
            for r in c_start.row..=c_end.row {
                for c in c_start.col..=c_end.col {
                    grid.set(r, c, true);
                }
            }
            // Convert dot coords → char coords (row/4, col/2).
            let top_row = (c_start.row.max(0) as usize) / 4;
            let top_col = (c_start.col.max(0) as usize) / 2;
            let bot_row = (c_end.row.max(0) as usize) / 4;
            let bot_col = (c_end.col.max(0) as usize) / 2;
            CentreBounds {
                top_left: (top_row, top_col),
                bottom_right: (bot_row, bot_col),
            }
        } else {
            NO_CENTRE
        };

        // ── Initial head / tail (key nodes) ───────────────────────────────────
        // Head: `size` dots arranged vertically near the top of the grid, just
        //       right of the column midpoint (matches Go `key_nodes`).
        // Tail: `size` dots arranged horizontally near the row midpoint,
        //       starting at the left edge.
        // The initial arc segment connects tail[i] → head[i] for each i.
        let rem = (dot_cols % 2) + ((size - 2) / 2);
        let col_start = (dot_cols / 2 + rem) as isize;

        // Scale head row positions by 2 to account for dot_rows = 2*dot_cols.
        let head: Vec<Coord> = (0..sz).map(|n| Coord::new(n * 2, col_start)).collect();
        // Tail row also scaled: use col_start * 2 as the row (≈ row_mid).
        let tail_row = col_start * 2;
        let tail: Vec<Coord> = (0..sz).map(|n| Coord::new(tail_row, n)).collect();

        for i in 0..size {
            grid.fill(tail[i], head[i]);
        }

        let width = dot_cols as isize;
        let height = dot_rows as isize; // = 2 * width

        Self {
            grid,
            head,
            tail,
            // Always build CW maps; CCW is achieved by mirroring in render_lines.
            head_map: make_head_map(width, height, sz, false),
            tail_map: make_tail_map(width, height, sz, false),
            centre_bounds,
        }
    }

    /// Advance the arc one step.
    fn walk(&mut self) {
        step(&mut self.head, &self.head_map);
        for &pos in &self.head {
            self.grid.set(pos.row, pos.col, true);
        }
        let old_tail = self.tail.clone();
        step(&mut self.tail, &self.tail_map);
        for &pos in &old_tail {
            self.grid.set(pos.row, pos.col, false);
        }
    }

    /// Render the current grid into [`Line`]s.
    ///
    /// When `spin` is `CounterClockwise` each output row is reversed so the
    /// arc appears to travel counter-clockwise.  The centre-colour switching
    /// logic is mirrored accordingly.
    fn render_lines(
        &self,
        outer_color: Color,
        inner_color: Color,
        style: RectStyle,
        spin: Spin,
    ) -> Vec<Line<'static>> {
        let mut lines = if style.is_braille() {
            self.render_braille(outer_color, inner_color, style)
        } else {
            self.render_chars(outer_color, inner_color, style)
        };
        if matches!(spin, Spin::CounterClockwise) {
            for line in &mut lines {
                line.spans.reverse();
            }
        }
        lines
    }

    fn render_braille(
        &self,
        outer_color: Color,
        inner_color: Color,
        style: RectStyle,
    ) -> Vec<Line<'static>> {
        let total_rows = self.grid.cells.len();
        let total_cols = self.grid.dot_cols;
        let char_rows = total_rows.div_ceil(4);
        let char_cols = total_cols.div_ceil(2);

        // Pack dot grid into braille bytes.
        let mut screen: Vec<Vec<u8>> = vec![vec![0u8; char_cols]; char_rows];
        for (row, row_cells) in self.grid.cells.iter().enumerate() {
            for (col, &on) in row_cells.iter().enumerate() {
                if !on {
                    continue;
                }
                let ci = row / 4;
                let cj = col / 2;
                if ci < char_rows && cj < char_cols {
                    let bit = BRAILLE_MAP[row % 4][col % 2];
                    screen[ci][cj] |= 1 << bit;
                }
            }
        }

        // Emit lines, toggling colour at centre column boundaries.
        let mut lines = Vec::with_capacity(char_rows);
        for (i, row) in screen.iter().enumerate() {
            let mut spans = Vec::with_capacity(char_cols);
            let mut color = outer_color;
            for (j, &byte) in row.iter().enumerate() {
                if self.centre_bounds.should_switch(i, j) {
                    color = if color == outer_color {
                        inner_color
                    } else {
                        outer_color
                    };
                }
                let (arc_ch, empty_ch) = style.cell_chars(byte);
                let ch = if byte != 0 { arc_ch } else { empty_ch };
                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }
            lines.push(Line::from(spans));
        }
        lines
    }

    fn render_chars(
        &self,
        outer_color: Color,
        inner_color: Color,
        style: RectStyle,
    ) -> Vec<Line<'static>> {
        // Collapse the dot grid to per-char-cell on/off — any dot set in a
        // 2×4 block makes the whole char-cell "on".
        let total_rows = self.grid.cells.len();
        let total_cols = self.grid.dot_cols;
        let char_rows = total_rows.div_ceil(4);
        let char_cols = total_cols.div_ceil(2);

        let mut screen: Vec<Vec<bool>> = vec![vec![false; char_cols]; char_rows];
        for (row, row_cells) in self.grid.cells.iter().enumerate() {
            for (col, &on) in row_cells.iter().enumerate() {
                if on {
                    let ci = row / 4;
                    let cj = col / 2;
                    if ci < char_rows && cj < char_cols {
                        screen[ci][cj] = true;
                    }
                }
            }
        }

        let mut lines = Vec::with_capacity(char_rows);
        for (i, row) in screen.iter().enumerate() {
            let mut spans = Vec::with_capacity(char_cols);
            let mut color = outer_color;
            for (j, &on) in row.iter().enumerate() {
                if self.centre_bounds.should_switch(i, j) {
                    color = if color == outer_color {
                        inner_color
                    } else {
                        outer_color
                    };
                }
                let byte: u8 = if on { 1 } else { 0 };
                let (arc_ch, empty_ch) = style.cell_chars(byte);
                let ch = if on { arc_ch } else { empty_ch };
                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }
            lines.push(Line::from(spans));
        }
        lines
    }
}

// ── NarrowEngine — 1-char-wide Zed-style column spinner ──────────────────────
//
// A sliding arc window travels down (or up, for CCW) a single column of
// braille cells.
//
// Layout of one braille cell (2 dot-cols × 4 dot-rows):
//
//   bit0  bit3   row0
//   bit1  bit4   row1
//   bit2  bit5   row2
//   bit6  bit7   row3
//
// The 10 Zed cap frames trace a clockwise arc around the perimeter of one
// cell.  They are used for the leading and trailing rows of the arc window;
// interior rows are always fully lit (⣿).
//
//   frame 0: ⠋  frame 1: ⠙  frame 2: ⠹  frame 3: ⠸  frame 4: ⠼
//   frame 5: ⠴  frame 6: ⠦  frame 7: ⠧  frame 8: ⠇  frame 9: ⠏

/// The 10 Zed braille cap bytes, clockwise from top-left.
const ZED_FRAMES: [u8; 10] = [0x0B, 0x19, 0x39, 0x38, 0x3C, 0x34, 0x26, 0x27, 0x07, 0x0F];

/// 1-char-wide sliding-arc column renderer.
#[derive(Debug, Clone)]
struct NarrowEngine {
    /// Total height of the column in character rows.
    height: usize,
    /// Number of arc rows in the lit window (≥ 2).
    arc_len: usize,
    /// Index of the head row (front of the arc, 0 = top row).
    head: usize,
    /// Phase within the 10-frame cap cycle for the head boundary cell.
    cap_phase: usize,
}

impl NarrowEngine {
    fn build(height_chars: usize, spin: Spin) -> Self {
        let height = height_chars.max(3);
        let arc_len = (height / 4).max(2).min(height);
        // For counter-clockwise, start the head at the bottom.
        let head = if matches!(spin, Spin::CounterClockwise) {
            height - 1
        } else {
            0
        };
        Self {
            height,
            arc_len,
            head,
            cap_phase: 0,
        }
    }

    /// Advance one step.
    fn walk(&mut self, spin: Spin) {
        let n = self.height;
        if matches!(spin, Spin::CounterClockwise) {
            self.head = (self.head + n - 1) % n;
        } else {
            self.head = (self.head + 1) % n;
        }
        self.cap_phase = (self.cap_phase + 1) % ZED_FRAMES.len();
    }

    fn render_lines(&self, outer_color: Color, dim_color: Color) -> Vec<Line<'static>> {
        let n = self.height;
        let tail = (self.head + n - (self.arc_len - 1)) % n;

        let head_byte = ZED_FRAMES[self.cap_phase];
        let tail_byte = ZED_FRAMES[(self.cap_phase + 5) % ZED_FRAMES.len()];

        let mut lines = Vec::with_capacity(n);
        for row in 0..n {
            let (byte, color) = if row == self.head {
                (head_byte, outer_color)
            } else if row == tail {
                (tail_byte, outer_color)
            } else {
                let in_arc = if self.head >= tail {
                    row > tail && row < self.head
                } else {
                    row > tail || row < self.head
                };
                if in_arc {
                    (0xFFu8, outer_color)
                } else {
                    (0x00u8, dim_color)
                }
            };
            let ch = if byte == 0 {
                '\u{2800}'
            } else {
                char::from_u32(BRAILLE_BASE + u32::from(byte)).unwrap_or('\u{2800}')
            };
            lines.push(Line::from(Span::styled(
                ch.to_string(),
                Style::default().fg(color),
            )));
        }
        lines
    }
}

// ── Public widget ─────────────────────────────────────────────────────────────

/// A braille-arc spinner that travels around a rectangle.
///
/// Choose between [`RectShape::Square`] (a square character-cell spinner) and
/// [`RectShape::Narrow`] (Zed-style 1-char-wide sidebar arc).
///
/// Control the spin direction with [`Spin::Clockwise`] (default) or
/// [`Spin::CounterClockwise`].
///
/// Control the centre with [`Centre::Filled`] or [`Centre::Empty`].
///
/// # Quick start
///
/// ```no_run
/// use ratatui::style::Color;
/// use tui_spinner::{Centre, RectShape, RectSpinner, RectStyle, Spin};
///
/// // Classic filled square, clockwise
/// let sq = RectSpinner::new(42)
///     .shape(RectShape::Square(2))
///     .render_style(RectStyle::Arc)
///     .outer_color(Color::Cyan)
///     .inner_color(Color::DarkGray)
///     .centre(Centre::Filled);
///
/// // Counter-clockwise hollow square
/// let ccw = RectSpinner::new(42)
///     .shape(RectShape::Square(3))
///     .spin(Spin::CounterClockwise)
///     .centre(Centre::Empty)
///     .outer_color(Color::Green);
///
/// // Narrow sidebar spinner
/// let narrow = RectSpinner::new(42)
///     .shape(RectShape::Narrow(10))
///     .outer_color(Color::Green);
/// ```
#[derive(Debug, Clone)]
pub struct RectSpinner<'a> {
    tick: u64,
    shape: RectShape,
    spin: Spin,
    ticks_per_step: u64,
    rect_style: RectStyle,
    outer_color: Color,
    inner_color: Color,
    centre: Centre,
    block: Option<Block<'a>>,
    style: Style,
    alignment: Alignment,
}

impl<'a> RectSpinner<'a> {
    /// Creates a new [`RectSpinner`] with defaults:
    /// `Square(2)` shape, `Arc` style, clockwise spin, white outer,
    /// dark-grey inner, filled centre, 1 tick per step.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::RectSpinner;
    ///
    /// let spinner = RectSpinner::new(42);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            shape: RectShape::Square(2),
            spin: Spin::Clockwise,
            ticks_per_step: 1,
            rect_style: RectStyle::Arc,
            outer_color: Color::White,
            inner_color: Color::DarkGray,
            centre: Centre::Filled,
            block: None,
            style: Style::default(),
            alignment: Alignment::Left,
        }
    }

    /// Sets the spinner shape (default: `Square(2)`).
    #[must_use]
    pub const fn shape(mut self, shape: RectShape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the spin direction (default: `Clockwise`).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{RectSpinner, Spin};
    ///
    /// let ccw = RectSpinner::new(0).spin(Spin::CounterClockwise);
    /// ```
    #[must_use]
    pub const fn spin(mut self, spin: Spin) -> Self {
        self.spin = spin;
        self
    }

    /// Sets the rendering style (default: `Arc`).
    #[must_use]
    pub const fn render_style(mut self, style: RectStyle) -> Self {
        self.rect_style = style;
        self
    }

    /// Sets the colour of the rotating arc (default: `Color::White`).
    #[must_use]
    pub const fn outer_color(mut self, color: Color) -> Self {
        self.outer_color = color;
        self
    }

    /// Sets the colour of the filled centre region (default: `Color::DarkGray`).
    ///
    /// Only visible when [`Centre::Filled`] is active.
    #[must_use]
    pub const fn inner_color(mut self, color: Color) -> Self {
        self.inner_color = color;
        self
    }

    /// Controls whether the centre is filled or empty (default: `Filled`).
    #[must_use]
    pub const fn centre(mut self, centre: Centre) -> Self {
        self.centre = centre;
        self
    }

    /// Sets how many ticks the arc holds each position before advancing
    /// (default: 1, higher = slower).
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

    /// Sets the base style for the widget area.
    #[must_use]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the horizontal alignment of the rendered lines (default: left).
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn build_square_engine(&self, size: usize) -> ArcEngine {
        let mut engine = ArcEngine::build_square(size, self.centre, self.spin);
        #[allow(clippy::cast_possible_truncation)]
        let steps = (self.tick / self.ticks_per_step) as usize;
        // Always walk CW; CCW is achieved by mirroring in render_lines.
        for _ in 0..steps {
            engine.walk();
        }
        engine
    }

    fn build_narrow_engine(&self, height: usize) -> NarrowEngine {
        let mut engine = NarrowEngine::build(height, self.spin);
        #[allow(clippy::cast_possible_truncation)]
        let steps = (self.tick / self.ticks_per_step) as usize;
        for _ in 0..steps {
            engine.walk(self.spin);
        }
        engine
    }
}

impl Styled for RectSpinner<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for RectSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &RectSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        let inner = if let Some(ref block) = self.block {
            let ia = block.inner(area);
            block.clone().render(area, buf);
            ia
        } else {
            area
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let lines = match self.shape {
            RectShape::Square(size) => {
                let engine = self.build_square_engine(size);
                engine.render_lines(
                    self.outer_color,
                    self.inner_color,
                    self.rect_style,
                    self.spin,
                )
            }
            RectShape::Narrow(height) => {
                let engine = self.build_narrow_engine(height);
                engine.render_lines(self.outer_color, self.inner_color)
            }
        };

        Paragraph::new(lines)
            .alignment(self.alignment)
            .render(inner, buf);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    // ── Geometry helpers ──────────────────────────────────────────────────────

    #[test]
    fn calc_dot_cols_values() {
        assert_eq!(calc_dot_cols(2), 8);
        assert_eq!(calc_dot_cols(3), 13);
        assert_eq!(calc_dot_cols(4), 18);
    }

    #[test]
    fn calc_dot_rows_is_double_cols() {
        for size in 2..=6 {
            assert_eq!(calc_dot_rows(size), 2 * calc_dot_cols(size));
        }
    }

    #[test]
    fn char_output_is_square() {
        // After braille packing (÷2 cols, ÷4 rows) the char grid must be square.
        // With dot_rows = 2*dot_cols and no offset: char_cols = dot_cols/2,
        // char_rows = 2*dot_cols/4 = dot_cols/2 — always equal.
        for size in 2..=6 {
            let dot_cols = calc_dot_cols(size);
            let dot_rows = calc_dot_rows(size);
            // vertical_offset is 0 for all sizes now.
            let char_cols = dot_cols.div_ceil(2);
            let char_rows = dot_rows.div_ceil(4);
            assert_eq!(
                char_cols, char_rows,
                "size {size}: char grid should be square ({char_cols}×{char_rows})"
            );
        }
    }

    // ── Square engine ─────────────────────────────────────────────────────────

    #[test]
    fn square_engine_builds_all_sizes() {
        for size in 2..=6 {
            for centre in [Centre::Filled, Centre::Empty] {
                for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                    let _ = ArcEngine::build_square(size, centre, spin);
                }
            }
        }
    }

    #[test]
    fn square_engine_walk_does_not_panic() {
        let mut e = ArcEngine::build_square(2, Centre::Filled, Spin::Clockwise);
        let dim = calc_dot_cols(2);
        for _ in 0..dim * 8 {
            e.walk();
        }
    }

    #[test]
    fn square_engine_ccw_walk_does_not_panic() {
        let mut e = ArcEngine::build_square(2, Centre::Filled, Spin::CounterClockwise);
        let dim = calc_dot_cols(2);
        for _ in 0..dim * 8 {
            e.walk();
        }
    }

    #[test]
    fn square_engine_advances_after_walk() {
        let e0 = ArcEngine::build_square(2, Centre::Filled, Spin::Clockwise);
        let mut e1 = ArcEngine::build_square(2, Centre::Filled, Spin::Clockwise);
        e1.walk();
        let l0 = e0.render_lines(
            Color::White,
            Color::DarkGray,
            RectStyle::Arc,
            Spin::Clockwise,
        );
        let l1 = e1.render_lines(
            Color::White,
            Color::DarkGray,
            RectStyle::Arc,
            Spin::Clockwise,
        );
        assert_ne!(l0, l1, "grid should differ after one walk");
    }

    #[test]
    fn cw_and_ccw_differ() {
        // Both engines are built identically (CW walk); CCW is produced by
        // mirroring each row.  At step 0 the arc is not symmetric, so CW and
        // CCW must differ.
        let e = ArcEngine::build_square(2, Centre::Empty, Spin::Clockwise);
        let lcw = e.render_lines(
            Color::White,
            Color::DarkGray,
            RectStyle::Arc,
            Spin::Clockwise,
        );
        let lccw = e.render_lines(
            Color::White,
            Color::DarkGray,
            RectStyle::Arc,
            Spin::CounterClockwise,
        );
        assert_ne!(lcw, lccw, "CW and CCW should produce different frames");
    }

    #[test]
    fn square_filled_vs_empty_differ() {
        let ef = ArcEngine::build_square(2, Centre::Filled, Spin::Clockwise);
        let ee = ArcEngine::build_square(2, Centre::Empty, Spin::Clockwise);
        let lf = ef.render_lines(
            Color::White,
            Color::DarkGray,
            RectStyle::Arc,
            Spin::Clockwise,
        );
        let le = ee.render_lines(
            Color::White,
            Color::DarkGray,
            RectStyle::Arc,
            Spin::Clockwise,
        );
        assert_ne!(lf, le, "filled and empty centres should render differently");
    }

    #[test]
    fn square_render_row_count_size2() {
        let e = ArcEngine::build_square(2, Centre::Filled, Spin::Clockwise);
        let lines = e.render_lines(
            Color::White,
            Color::DarkGray,
            RectStyle::Arc,
            Spin::Clockwise,
        );
        let dot_cols = calc_dot_cols(2);
        let char_cols = dot_cols.div_ceil(2);
        assert_eq!(
            lines.len(),
            char_cols,
            "row count should equal char_cols for square output"
        );
    }

    #[test]
    fn square_render_row_count_size3() {
        let e = ArcEngine::build_square(3, Centre::Filled, Spin::Clockwise);
        let lines = e.render_lines(
            Color::White,
            Color::DarkGray,
            RectStyle::Arc,
            Spin::Clockwise,
        );
        let char_cols = calc_dot_cols(3).div_ceil(2);
        assert_eq!(lines.len(), char_cols);
    }

    // ── NarrowEngine ──────────────────────────────────────────────────────────

    #[test]
    fn narrow_engine_builds() {
        for h in [3, 4, 6, 8, 12, 20] {
            for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                let _ = NarrowEngine::build(h, spin);
            }
        }
    }

    #[test]
    fn narrow_engine_walk_does_not_panic() {
        for spin in [Spin::Clockwise, Spin::CounterClockwise] {
            let mut e = NarrowEngine::build(8, spin);
            for _ in 0..200 {
                e.walk(spin);
            }
        }
    }

    #[test]
    fn narrow_engine_advances_after_walk() {
        let e0 = NarrowEngine::build(8, Spin::Clockwise);
        let mut e1 = NarrowEngine::build(8, Spin::Clockwise);
        e1.walk(Spin::Clockwise);
        let l0 = e0.render_lines(Color::White, Color::DarkGray);
        let l1 = e1.render_lines(Color::White, Color::DarkGray);
        assert_ne!(l0, l1, "narrow should differ after one walk");
    }

    #[test]
    fn narrow_render_has_one_char_per_row() {
        let e = NarrowEngine::build(8, Spin::Clockwise);
        let lines = e.render_lines(Color::White, Color::DarkGray);
        assert_eq!(lines.len(), 8, "should have one line per row");
        for line in &lines {
            assert_eq!(line.spans.len(), 1, "each row is exactly 1 braille char");
        }
    }

    #[test]
    fn narrow_render_row_count_matches_height() {
        for h in [3, 6, 10, 15] {
            let e = NarrowEngine::build(h, Spin::Clockwise);
            let lines = e.render_lines(Color::White, Color::DarkGray);
            assert_eq!(lines.len(), h, "row count should equal height {h}");
        }
    }

    #[test]
    fn narrow_arc_rows_are_full_braille() {
        let full = char::from_u32(BRAILLE_BASE + 0xFF).unwrap();
        let e20 = NarrowEngine::build(20, Spin::Clockwise);
        let lines20 = e20.render_lines(Color::Cyan, Color::DarkGray);
        let full20 = lines20
            .iter()
            .filter(|l| l.spans[0].content.chars().next() == Some(full))
            .count();
        // arc_len for height=20 is max(20/4,2)=5 → 3 interior rows.
        assert!(
            full20 >= 3,
            "expected ≥3 interior fully-lit rows for height=20"
        );
    }

    #[test]
    fn narrow_wraps_after_full_revolution() {
        fn gcd(a: usize, b: usize) -> usize {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }
        fn lcm(a: usize, b: usize) -> usize {
            a / gcd(a, b) * b
        }

        for height in [5, 8, 10, 20] {
            let mut e = NarrowEngine::build(height, Spin::Clockwise);
            let period = lcm(e.height, ZED_FRAMES.len());
            let l0 = e.render_lines(Color::Cyan, Color::DarkGray);
            for _ in 0..period {
                e.walk(Spin::Clockwise);
            }
            let ln = e.render_lines(Color::Cyan, Color::DarkGray);
            assert_eq!(
                l0, ln,
                "height={height}: should return to same frame after {period} steps"
            );
        }
    }

    // ── Widget rendering ──────────────────────────────────────────────────────

    #[test]
    fn widget_square_renders_without_panic() {
        for size in [2, 3, 4] {
            for style in [
                RectStyle::Arc,
                RectStyle::Dense,
                RectStyle::Shade,
                RectStyle::Pixel,
            ] {
                for centre in [Centre::Filled, Centre::Empty] {
                    for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                        let spinner = RectSpinner::new(10)
                            .shape(RectShape::Square(size))
                            .render_style(style)
                            .centre(centre)
                            .spin(spin);
                        let area = Rect::new(0, 0, 30, 15);
                        let mut buf = Buffer::empty(area);
                        Widget::render(&spinner, area, &mut buf);
                    }
                }
            }
        }
    }

    #[test]
    fn widget_narrow_renders_without_panic() {
        for tick in [0u64, 1, 5, 100] {
            for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                let spinner = RectSpinner::new(tick)
                    .shape(RectShape::Narrow(10))
                    .spin(spin);
                let area = Rect::new(0, 0, 5, 10);
                let mut buf = Buffer::empty(area);
                Widget::render(&spinner, area, &mut buf);
            }
        }
    }

    #[test]
    fn widget_narrow_different_ticks_differ() {
        let area = Rect::new(0, 0, 5, 10);
        let mut b0 = Buffer::empty(area);
        let mut b1 = Buffer::empty(area);
        Widget::render(
            &RectSpinner::new(0).shape(RectShape::Narrow(10)),
            area,
            &mut b0,
        );
        Widget::render(
            &RectSpinner::new(3).shape(RectShape::Narrow(10)),
            area,
            &mut b1,
        );
        assert_ne!(b0, b1, "different ticks should produce different output");
    }

    #[test]
    fn widget_zero_area_no_panic() {
        let spinner = RectSpinner::new(0);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        Widget::render(&spinner, area, &mut buf);
    }

    #[test]
    fn different_ticks_produce_different_output() {
        let area = Rect::new(0, 0, 20, 10);
        let mut b0 = Buffer::empty(area);
        let mut b5 = Buffer::empty(area);
        Widget::render(&RectSpinner::new(0), area, &mut b0);
        Widget::render(&RectSpinner::new(5), area, &mut b5);
        assert_ne!(b0, b5, "tick 0 and tick 5 should render differently");
    }

    #[test]
    fn all_rect_styles_render_without_panic() {
        let styles = [
            RectStyle::Arc,
            RectStyle::Dense,
            RectStyle::Shade,
            RectStyle::Outline,
            RectStyle::Dot,
            RectStyle::Star,
            RectStyle::Diamond,
            RectStyle::Cross,
            RectStyle::Fade,
            RectStyle::Pixel,
        ];
        for style in styles {
            let spinner = RectSpinner::new(7).render_style(style);
            let area = Rect::new(0, 0, 20, 10);
            let mut buf = Buffer::empty(area);
            Widget::render(&spinner, area, &mut buf);
        }
    }

    #[test]
    fn cell_chars_arc_zero_byte() {
        let (_, empty) = RectStyle::Arc.cell_chars(0);
        assert_eq!(empty, '\u{2800}');
    }

    #[test]
    fn cell_chars_shade() {
        let (arc, empty) = RectStyle::Shade.cell_chars(1);
        assert_eq!(arc, '█');
        assert_eq!(empty, '░');
    }

    #[test]
    fn cell_chars_fade_gradient() {
        let (c0, _) = RectStyle::Fade.cell_chars(0b0000_0000);
        let (c1, _) = RectStyle::Fade.cell_chars(0b0000_0001);
        let (c2, _) = RectStyle::Fade.cell_chars(0b0001_1111);
        let (c3, _) = RectStyle::Fade.cell_chars(0b1111_1111);
        assert_eq!(c0, '░');
        assert_eq!(c1, '▒');
        assert_eq!(c2, '▓');
        assert_eq!(c3, '█');
    }

    #[test]
    fn is_braille_classification() {
        assert!(RectStyle::Arc.is_braille());
        assert!(RectStyle::Dense.is_braille());
        assert!(RectStyle::Fade.is_braille());
        assert!(!RectStyle::Shade.is_braille());
        assert!(!RectStyle::Outline.is_braille());
        assert!(!RectStyle::Pixel.is_braille());
    }

    #[test]
    fn centre_bounds_should_switch() {
        let cb = CentreBounds {
            top_left: (1, 2),
            bottom_right: (3, 5),
        };
        assert!(cb.should_switch(1, 2));
        assert!(cb.should_switch(2, 2));
        assert!(cb.should_switch(3, 5));
        assert!(!cb.should_switch(2, 3));
        assert!(!cb.should_switch(0, 2));
        assert!(!cb.should_switch(4, 2));
    }

    #[test]
    fn no_centre_never_switches() {
        assert!(!NO_CENTRE.should_switch(0, 0));
        assert!(!NO_CENTRE.should_switch(100, 100));
    }

    #[test]
    fn spin_default_is_clockwise() {
        assert_eq!(Spin::default(), Spin::Clockwise);
    }
}
