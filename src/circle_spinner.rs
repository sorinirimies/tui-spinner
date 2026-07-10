//! Circle braille-arc spinner.
//!
//! A comet-like arc rotates around a circular braille-dot ring.
//!
//! ## How it works
//!
//! 1. The circle perimeter is computed with the midpoint circle algorithm.
//!    Dot coordinates are stored as `(row, col)` in a 1:1 dot pitch — one
//!    braille dot column is one unit wide, one braille dot row is one unit
//!    tall.  Because each braille character packs 2 dot-cols horizontally
//!    and 4 dot-rows vertically, and terminal cells are ~2× taller than
//!    wide, these two factors cancel exactly: 1 dot-col pixel width =
//!    `cell_w/2`, 1 dot-row pixel height = `cell_h/4` = `cell_w/2`.  So a 1:1
//!    dot pitch produces a visually round circle.
//!
//! 2. The perimeter dots are sorted clockwise (12-o'clock first).
//!
//! 3. A boolean dot-grid is allocated to the bounding box of the circle.
//!    No interior fill — only the perimeter ring is ever drawn.
//!
//! 4. Head and tail indices step through the perimeter list each `walk()`
//!    call: head sets its dot `true`, tail clears its dot `false`.  This
//!    produces the travelling comet arc identical to [`crate::RectSpinner`].
//!
//! 5. The grid is packed into braille bytes and rendered as [`Line`]s.
//!    Arc dots use `arc_color`; the dim remainder of the ring uses
//!    `dim_color`.

use std::collections::HashSet;

use crate::rect_spinner::Spin;

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Widget};

// ── Braille constants ─────────────────────────────────────────────────────────

const BRAILLE_BASE: u32 = 0x2800;

/// Bit index within a braille byte, indexed by `(dot_row % 4, dot_col % 2)`.
const BRAILLE_MAP: [[u8; 2]; 4] = [
    [0, 3], // row 0: left→bit 0,  right→bit 3
    [1, 4], // row 1: left→bit 1,  right→bit 4
    [2, 5], // row 2: left→bit 2,  right→bit 5
    [6, 7], // row 3: left→bit 6,  right→bit 7
];

// ── Dot ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Dot {
    row: isize,
    col: isize,
}

impl Dot {
    const fn new(row: isize, col: isize) -> Self {
        Self { row, col }
    }
}

// ── Perimeter ─────────────────────────────────────────────────────────────────

/// Compute all dot positions on a circle perimeter using the midpoint circle
/// algorithm at 1:1 dot pitch, then sort them clockwise from 12 o'clock.
#[allow(clippy::cast_possible_wrap)]
fn circle_perimeter(r: usize) -> Vec<Dot> {
    if r == 0 {
        return vec![Dot::new(0, 0)];
    }

    let ri = r as isize;
    let mut points: HashSet<(isize, isize)> = HashSet::new();

    let mut x: isize = 0;
    let mut y: isize = ri;
    let mut d: isize = 1 - ri;

    while x <= y {
        // All 8 octant reflections — no column scaling, pure 1:1 pitch.
        for &(px, py) in &[
            (x, -y),
            (y, -x),
            (y, x),
            (x, y),
            (-x, y),
            (-y, x),
            (-y, -x),
            (-x, -y),
        ] {
            points.insert((py, px)); // (row, col) — no ×2
        }
        if d < 0 {
            d += 2 * x + 3;
        } else {
            d += 2 * (x - y) + 5;
            y -= 1;
        }
        x += 1;
    }

    sort_clockwise(points.into_iter().collect())
}

/// Sort `(row, col)` dot coords into clockwise order starting from 12 o'clock.
#[allow(clippy::cast_precision_loss)]
fn sort_clockwise(dots: Vec<(isize, isize)>) -> Vec<Dot> {
    if dots.is_empty() {
        return vec![];
    }

    let n = dots.len() as f64;
    let cr = dots.iter().map(|&(r, _)| r as f64).sum::<f64>() / n;
    let cc = dots.iter().map(|&(_, c)| c as f64).sum::<f64>() / n;

    let mut with_angle: Vec<(f64, isize, isize)> = dots
        .into_iter()
        .map(|(r, c)| {
            let dr = -(r as f64 - cr);
            let dc = c as f64 - cc;
            let raw = dc.atan2(dr); // 0 = top, increases clockwise
            let angle = if raw < 0.0 {
                raw + 2.0 * std::f64::consts::PI
            } else {
                raw
            };
            (angle, r, c)
        })
        .collect();

    with_angle.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    with_angle
        .into_iter()
        .map(|(_, r, c)| Dot::new(r, c))
        .collect()
}

// ── Arc engine ────────────────────────────────────────────────────────────────

/// All state for one animation frame of a circle spinner.
#[derive(Debug, Clone)]
struct CircleEngine {
    /// Boolean dot-grid; dimensions `dot_rows × dot_cols`.
    cells: Vec<Vec<bool>>,
    dot_rows: usize,
    dot_cols: usize,
    /// Offset to map perimeter-space coords → grid indices.
    row_offset: isize,
    col_offset: isize,
    /// Sorted clockwise perimeter dot list.
    perimeter: Vec<Dot>,
    /// Index of the arc front (most recently lit dot).
    head: usize,
    /// Index of the arc back (next dot to be erased).
    tail: usize,
    /// Number of perimeter dots in the lit arc.
    arc_len: usize,
}

impl CircleEngine {
    #[allow(clippy::cast_sign_loss)]
    fn build(radius: usize, arc_len_override: usize, spin: Spin) -> Self {
        let perimeter = circle_perimeter(radius);
        let n = perimeter.len();

        let min_row = perimeter.iter().map(|d| d.row).min().unwrap_or(0);
        let max_row = perimeter.iter().map(|d| d.row).max().unwrap_or(0);
        let min_col = perimeter.iter().map(|d| d.col).min().unwrap_or(0);
        let max_col = perimeter.iter().map(|d| d.col).max().unwrap_or(0);

        let row_offset = -min_row;
        let col_offset = -min_col;
        let dot_rows = (max_row - min_row + 1) as usize;
        let dot_cols = (max_col - min_col + 1) as usize;

        let cells = vec![vec![false; dot_cols]; dot_rows];

        let arc_len = if arc_len_override > 0 {
            arc_len_override.min(n)
        } else {
            (n / 4).max(1)
        };

        // Head starts at index 0 (top of circle, clockwise) or index n-1
        // (top of circle, counter-clockwise); tail trails arc_len behind.
        let head = 0usize;
        let tail = if matches!(spin, Spin::CounterClockwise) {
            arc_len % n
        } else {
            (n - arc_len) % n
        };

        let mut engine = Self {
            cells,
            dot_rows,
            dot_cols,
            row_offset,
            col_offset,
            perimeter,
            head,
            tail,
            arc_len,
        };

        // Light the initial arc.
        for i in 0..arc_len {
            let dot = engine.perimeter[i];
            engine.set_dot(dot, true);
        }

        engine
    }

    #[inline]
    #[allow(clippy::cast_sign_loss)]
    fn set_dot(&mut self, dot: Dot, value: bool) {
        let r = (dot.row + self.row_offset) as usize;
        let c = (dot.col + self.col_offset) as usize;
        if r < self.dot_rows && c < self.dot_cols {
            self.cells[r][c] = value;
        }
    }

    /// Advance one step: head lights next dot, tail erases its dot.
    fn walk(&mut self, spin: Spin) {
        let n = self.perimeter.len();

        if matches!(spin, Spin::CounterClockwise) {
            self.head = (self.head + n - 1) % n;
            let new_head = self.perimeter[self.head];
            self.set_dot(new_head, true);

            let old_tail = self.perimeter[self.tail];
            self.set_dot(old_tail, false);
            self.tail = (self.tail + n - 1) % n;
        } else {
            self.head = (self.head + 1) % n;
            let new_head = self.perimeter[self.head];
            self.set_dot(new_head, true);

            let old_tail = self.perimeter[self.tail];
            self.set_dot(old_tail, false);
            self.tail = (self.tail + 1) % n;
        }
    }

    /// Render the grid into braille [`Line`]s.
    ///
    /// Arc dots → `arc_color`, dim ring dots → `dim_color`.
    #[allow(clippy::cast_sign_loss)]
    fn render_lines(&self, arc_color: Color, dim_color: Color) -> Vec<Line<'static>> {
        let char_rows = self.dot_rows.div_ceil(4);
        let char_cols = self.dot_cols.div_ceil(2);

        // Build the set of currently-lit arc dot positions for fast lookup.
        let arc_set: HashSet<(isize, isize)> = (0..self.arc_len)
            .map(|i| {
                let idx = (self.tail + i) % self.perimeter.len();
                let d = self.perimeter[idx];
                (d.row, d.col)
            })
            .collect();

        // Two braille byte-grids: lit arc and dim ring.
        let mut bright: Vec<Vec<u8>> = vec![vec![0u8; char_cols]; char_rows];
        let mut dim: Vec<Vec<u8>> = vec![vec![0u8; char_cols]; char_rows];

        for dot in &self.perimeter {
            let r = (dot.row + self.row_offset) as usize;
            let c = (dot.col + self.col_offset) as usize;
            if r >= self.dot_rows || c >= self.dot_cols {
                continue;
            }
            let ci = r / 4;
            let cj = c / 2;
            if ci >= char_rows || cj >= char_cols {
                continue;
            }
            let bit = BRAILLE_MAP[r % 4][c % 2];
            if arc_set.contains(&(dot.row, dot.col)) {
                bright[ci][cj] |= 1 << bit;
            } else {
                dim[ci][cj] |= 1 << bit;
            }
        }

        // Compose: bright > dim > blank.
        let mut lines = Vec::with_capacity(char_rows);
        for ri in 0..char_rows {
            let mut spans = Vec::with_capacity(char_cols);
            for ci in 0..char_cols {
                let b = bright[ri][ci];
                let d = dim[ri][ci];
                let (byte, color) = if b != 0 {
                    (b, arc_color)
                } else if d != 0 {
                    (d, dim_color)
                } else {
                    (0u8, dim_color)
                };
                let ch = if byte == 0 {
                    '\u{2800}'
                } else {
                    char::from_u32(BRAILLE_BASE + u32::from(byte)).unwrap_or('\u{2800}')
                };
                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }
            lines.push(Line::from(spans));
        }
        lines
    }
}

// ── Public widget ─────────────────────────────────────────────────────────────

/// A spinner whose arc rotates clockwise around a circular braille-dot ring.
///
/// Uses a 1:1 dot pitch which, after braille packing (2 dot-cols per char-col,
/// 4 dot-rows per char-row) and the ~2× terminal cell aspect ratio, produces a
/// visually round circle.
///
/// There is no centre fill — only the rotating arc on the ring is drawn.
///
/// # Layout
///
/// The rendered size in terminal characters is approximately:
///   - columns: `⌈(2r + 1) / 2⌉`
///   - rows:    `⌈(2r + 1) / 4⌉`
///
/// Use [`CircleSpinner::char_size`] to query the exact dimensions.
///
/// # Examples
///
/// ```no_run
/// use ratatui::style::Color;
/// use ratatui::Frame;
/// use ratatui::layout::Rect;
/// use tui_spinner::CircleSpinner;
///
/// fn draw(frame: &mut Frame, area: Rect, tick: u64) {
///     frame.render_widget(
///         CircleSpinner::new(tick)
///             .radius(6)
///             .arc_color(Color::Cyan)
///             .dim_color(Color::DarkGray),
///         area,
///     );
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CircleSpinner<'a> {
    tick: u64,
    radius: usize,
    /// Explicit arc length in perimeter dots (0 = auto ¼ of perimeter).
    arc_len: usize,
    ticks_per_step: u64,
    spin: Spin,
    /// Colour of the rotating bright arc.
    arc_color: Color,
    /// Colour of the dim remainder of the ring.
    dim_color: Color,
    block: Option<Block<'a>>,
    style: Style,
    alignment: Alignment,
}

impl<'a> CircleSpinner<'a> {
    /// Creates a new [`CircleSpinner`] with defaults: radius 4, clockwise spin,
    /// white arc, dark-gray dim ring, 1 tick per step, auto arc length.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::CircleSpinner;
    ///
    /// let spinner = CircleSpinner::new(42);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            radius: 4,
            arc_len: 0,
            ticks_per_step: 1,
            spin: Spin::Clockwise,
            arc_color: Color::White,
            dim_color: Color::DarkGray,
            block: None,
            style: Style::default(),
            alignment: Alignment::Left,
        }
    }

    /// Sets the circle radius in braille dots (default: 4, minimum: 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::CircleSpinner;
    ///
    /// let big = CircleSpinner::new(0).radius(8);
    /// ```
    #[must_use]
    pub fn radius(mut self, r: usize) -> Self {
        self.radius = r.max(1);
        self
    }

    /// Sets the number of perimeter dots in the bright arc
    /// (0 = auto ¼ of the total perimeter length).
    #[must_use]
    pub fn arc_len(mut self, len: usize) -> Self {
        self.arc_len = len;
        self
    }

    /// Sets the spin direction (default: [`Spin::Clockwise`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{CircleSpinner, Spin};
    ///
    /// let ccw = CircleSpinner::new(0).spin(Spin::CounterClockwise);
    /// ```
    #[must_use]
    pub const fn spin(mut self, spin: Spin) -> Self {
        self.spin = spin;
        self
    }

    /// Sets how many ticks each arc step is held (default: 1, higher = slower).
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.ticks_per_step = n.max(1);
        self
    }

    /// Sets the colour of the rotating bright arc (default: [`Color::White`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::CircleSpinner;
    ///
    /// let spinner = CircleSpinner::new(0).arc_color(Color::Cyan);
    /// ```
    #[must_use]
    pub const fn arc_color(mut self, color: Color) -> Self {
        self.arc_color = color;
        self
    }

    /// Sets the colour of the dim background ring (default: [`Color::DarkGray`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::CircleSpinner;
    ///
    /// let spinner = CircleSpinner::new(0).dim_color(Color::DarkGray);
    /// ```
    #[must_use]
    pub const fn dim_color(mut self, color: Color) -> Self {
        self.dim_color = color;
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
    /// use tui_spinner::CircleSpinner;
    ///
    /// let (cols, rows) = CircleSpinner::new(0).radius(4).char_size();
    /// // dot_cols = 2*4+1 = 9, char_cols = ceil(9/2) = 5
    /// // dot_rows = 2*4+1 = 9, char_rows = ceil(9/4) = 3
    /// assert_eq!(cols, 5);
    /// assert_eq!(rows, 3);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn char_size(&self) -> (u16, u16) {
        let dot_dim = self.radius * 2 + 1;
        let char_cols = dot_dim.div_ceil(2) as u16;
        let char_rows = dot_dim.div_ceil(4) as u16;
        (char_cols, char_rows)
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut engine = CircleEngine::build(self.radius, self.arc_len, self.spin);

        #[allow(clippy::cast_possible_truncation)]
        let steps = (self.tick / self.ticks_per_step) as usize;
        for _ in 0..steps {
            engine.walk(self.spin);
        }

        engine.render_lines(self.arc_color, self.dim_color)
    }
}

impl_styled_for!(CircleSpinner<'_>);

impl_to_text!(CircleSpinner<'_>, build_lines);

impl_widget_via_ref!(CircleSpinner<'_>);

impl Widget for &CircleSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_spinner_body!(self, area, buf, self.build_lines());
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rect_spinner::Spin;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    // ── Perimeter geometry ────────────────────────────────────────────────────

    #[test]
    fn circle_perimeter_has_points() {
        for r in 1..=8usize {
            let p = circle_perimeter(r);
            assert!(!p.is_empty(), "radius {r} should produce perimeter points");
        }
    }

    #[test]
    fn circle_perimeter_zero_returns_single_dot() {
        let p = circle_perimeter(0);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0], Dot::new(0, 0));
    }

    #[test]
    fn circle_perimeter_larger_radius_more_points() {
        assert!(circle_perimeter(6).len() > circle_perimeter(2).len());
    }

    #[test]
    fn circle_perimeter_is_square_bounding_box() {
        // With 1:1 pitch the bounding box should be (2r+1) × (2r+1).
        for r in 1..=6usize {
            let p = circle_perimeter(r);
            let min_row = p.iter().map(|d| d.row).min().unwrap();
            let max_row = p.iter().map(|d| d.row).max().unwrap();
            let min_col = p.iter().map(|d| d.col).min().unwrap();
            let max_col = p.iter().map(|d| d.col).max().unwrap();
            let height = (max_row - min_row + 1) as usize;
            let width = (max_col - min_col + 1) as usize;
            // Should be approximately square (within 1 dot either way).
            let diff = height.abs_diff(width);
            assert!(
                diff <= 2,
                "r={r}: bounding box {width}×{height} is not square"
            );
        }
    }

    #[test]
    fn circle_perimeter_sorted_clockwise() {
        let p = circle_perimeter(4);
        let cr = p.iter().map(|d| d.row as f64).sum::<f64>() / p.len() as f64;
        let cc = p.iter().map(|d| d.col as f64).sum::<f64>() / p.len() as f64;
        let angles: Vec<f64> = p
            .iter()
            .map(|d| {
                let dr = -(d.row as f64 - cr);
                let dc = d.col as f64 - cc;
                let raw = dc.atan2(dr);
                if raw < 0.0 {
                    raw + 2.0 * std::f64::consts::PI
                } else {
                    raw
                }
            })
            .collect();
        for w in angles.windows(2) {
            assert!(
                w[1] >= w[0] - 1e-9,
                "angles not monotone: {} > {}",
                w[0],
                w[1]
            );
        }
    }

    #[test]
    fn sort_clockwise_four_cardinals() {
        let pts = vec![(-1isize, 0isize), (0, 1), (1, 0), (0, -1)];
        let sorted = sort_clockwise(pts);
        assert_eq!(sorted[0], Dot::new(-1, 0), "first should be top");
        assert_eq!(sorted[1], Dot::new(0, 1), "second should be right");
        assert_eq!(sorted[2], Dot::new(1, 0), "third should be bottom");
        assert_eq!(sorted[3], Dot::new(0, -1), "fourth should be left");
    }

    // ── Engine ────────────────────────────────────────────────────────────────

    #[test]
    fn engine_builds_for_various_radii() {
        for r in [1, 2, 3, 4, 5, 8] {
            for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                let _ = CircleEngine::build(r, 0, spin);
            }
        }
    }

    #[test]
    fn engine_walk_does_not_panic() {
        for spin in [Spin::Clockwise, Spin::CounterClockwise] {
            let mut e = CircleEngine::build(4, 0, spin);
            for _ in 0..e.perimeter.len() * 2 {
                e.walk(spin);
            }
        }
    }

    #[test]
    fn engine_advances_after_walk() {
        let e0 = CircleEngine::build(4, 0, Spin::Clockwise);
        let mut e1 = CircleEngine::build(4, 0, Spin::Clockwise);
        e1.walk(Spin::Clockwise);
        let l0 = e0.render_lines(Color::Cyan, Color::DarkGray);
        let l1 = e1.render_lines(Color::Cyan, Color::DarkGray);
        assert_ne!(l0, l1, "frame should change after one walk step");
    }

    #[test]
    fn engine_wraps_after_full_revolution() {
        for spin in [Spin::Clockwise, Spin::CounterClockwise] {
            let mut e = CircleEngine::build(4, 0, spin);
            let n = e.perimeter.len();
            let l0 = e.render_lines(Color::Cyan, Color::DarkGray);
            for _ in 0..n {
                e.walk(spin);
            }
            let ln = e.render_lines(Color::Cyan, Color::DarkGray);
            assert_eq!(
                l0, ln,
                "should return to identical frame after full revolution ({spin:?})"
            );
        }
    }

    // ── Widget ────────────────────────────────────────────────────────────────

    #[test]
    fn widget_renders_without_panic() {
        for r in [1, 2, 4, 6, 8, 12] {
            for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                let spinner = CircleSpinner::new(10).radius(r).spin(spin);
                let area = Rect::new(0, 0, 40, 20);
                let mut buf = Buffer::empty(area);
                Widget::render(&spinner, area, &mut buf);
            }
        }
    }

    #[test]
    fn widget_zero_area_no_panic() {
        let spinner = CircleSpinner::new(0);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        Widget::render(&spinner, area, &mut buf);
    }

    #[test]
    fn different_ticks_produce_different_output() {
        let area = Rect::new(0, 0, 10, 5);
        let mut b0 = Buffer::empty(area);
        let mut b5 = Buffer::empty(area);
        Widget::render(
            &CircleSpinner::new(0).radius(4).spin(Spin::Clockwise),
            area,
            &mut b0,
        );
        Widget::render(
            &CircleSpinner::new(5).radius(4).spin(Spin::Clockwise),
            area,
            &mut b5,
        );
        assert_ne!(b0, b5, "tick 0 and tick 5 should render differently");
    }

    #[test]
    fn ticks_per_step_slows_animation() {
        // With ticks_per_step=3, tick=1 stays at step 0 — same frame as tick=0.
        let area = Rect::new(0, 0, 10, 5);
        let mut b0 = Buffer::empty(area);
        let mut b1 = Buffer::empty(area);
        Widget::render(
            &CircleSpinner::new(0).radius(4).ticks_per_step(3),
            area,
            &mut b0,
        );
        Widget::render(
            &CircleSpinner::new(1).radius(4).ticks_per_step(3),
            area,
            &mut b1,
        );
        assert_eq!(b0, b1, "slow spinner at tick 1 should equal tick 0");
    }

    #[test]
    fn cw_and_ccw_differ() {
        let area = Rect::new(0, 0, 10, 5);
        let mut bcw = Buffer::empty(area);
        let mut bccw = Buffer::empty(area);
        Widget::render(
            &CircleSpinner::new(5).radius(4).spin(Spin::Clockwise),
            area,
            &mut bcw,
        );
        Widget::render(
            &CircleSpinner::new(5).radius(4).spin(Spin::CounterClockwise),
            area,
            &mut bccw,
        );
        assert_ne!(bcw, bccw, "CW and CCW should produce different frames");
    }

    #[test]
    fn char_size_is_correct() {
        // radius 4: dot_dim = 2*4+1 = 9
        // char_cols = ceil(9/2) = 5, char_rows = ceil(9/4) = 3
        let (cols, rows) = CircleSpinner::new(0).radius(4).char_size();
        assert_eq!(cols, 5, "char cols for radius 4");
        assert_eq!(rows, 3, "char rows for radius 4");
    }

    #[test]
    fn char_size_radius_8() {
        // radius 8: dot_dim = 2*8+1 = 17
        // char_cols = ceil(17/2) = 9, char_rows = ceil(17/4) = 5
        let (cols, rows) = CircleSpinner::new(0).radius(8).char_size();
        assert_eq!(cols, 9, "char cols for radius 8");
        assert_eq!(rows, 5, "char rows for radius 8");
    }

    #[test]
    fn builder_chain_fields() {
        let s = CircleSpinner::new(10)
            .radius(6)
            .arc_len(5)
            .ticks_per_step(3)
            .spin(Spin::CounterClockwise)
            .arc_color(Color::Cyan)
            .dim_color(Color::DarkGray);
        assert_eq!(s.radius, 6);
        assert_eq!(s.arc_len, 5);
        assert_eq!(s.ticks_per_step, 3);
        assert_eq!(s.spin, Spin::CounterClockwise);
        assert_eq!(s.arc_color, Color::Cyan);
        assert_eq!(s.dim_color, Color::DarkGray);
    }

    #[test]
    fn ring_has_visible_content() {
        let spinner = CircleSpinner::new(0).radius(4).arc_color(Color::Cyan);
        let area = Rect::new(0, 0, 30, 10);
        let mut buf = Buffer::empty(area);
        Widget::render(&spinner, area, &mut buf);
        let has_content = buf
            .content()
            .iter()
            .any(|c| c.symbol() != " " && c.symbol() != "\u{2800}");
        assert!(
            has_content,
            "spinner should render some visible braille dots"
        );
    }

    #[test]
    fn to_lines_matches_build_lines() {
        let s = CircleSpinner::new(4).radius(3);
        assert_eq!(s.to_lines(), s.build_lines());
        assert_eq!(s.to_text().lines.len(), s.build_lines().len());
    }
}
