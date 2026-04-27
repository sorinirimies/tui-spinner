//! Rectangle braille-arc bouncing spinner.
//!
//! A Zed / Claude-style comet arc bounces back and forth along the perimeter
//! of a braille-dot rectangle.  Unlike [`crate::RectSpinner`] which rotates
//! continuously, this spinner **reverses direction** when the arc reaches
//! either end of the perimeter, producing a characteristic ping-pong effect.
//!
//! ## How it works
//!
//! 1. The perimeter of a `width × height` character rectangle is enumerated
//!    clockwise in 1:1 braille-dot coordinates: top edge → right edge →
//!    bottom edge → left edge.
//! 2. A sliding window of `arc_len` dots advances along the perimeter each
//!    step; when the leading edge reaches either end it reverses.
//! 3. The dot grid is packed into braille bytes and rendered as [`Line`]s.

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style, Styled};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::Spin;

// ── Braille constants ─────────────────────────────────────────────────────────

const BRAILLE_BASE: u32 = 0x2800;

/// `BRAILLE_MAP[dot_row % 4][dot_col % 2]` → bit index in the braille byte.
const BRAILLE_MAP: [[u8; 2]; 4] = [[0, 3], [1, 4], [2, 5], [6, 7]];

// ── Perimeter helper ──────────────────────────────────────────────────────────

/// Build the ordered clockwise perimeter of a `char_w × char_h` terminal
/// character rectangle in 1:1 braille-dot coordinates.
///
/// Returns `(dot_row, dot_col)` pairs, starting at the top-left corner and
/// travelling: top → right → bottom → left.
///
/// Total length = `2 * (char_w * 2 + char_h * 4) - 4`.
pub(crate) fn build_perimeter(char_w: usize, char_h: usize) -> Vec<(usize, usize)> {
    let dot_w = char_w * 2;
    let dot_h = char_h * 4;
    let capacity = 2 * (dot_w + dot_h) - 4;
    let mut perim = Vec::with_capacity(capacity);

    // Top edge: left → right
    for c in 0..dot_w {
        perim.push((0, c));
    }
    // Right edge: row 1 → bottom (top-right corner already included)
    for r in 1..dot_h {
        perim.push((r, dot_w - 1));
    }
    // Bottom edge: col dot_w-2 → 0 (bottom-right corner already included)
    for c in (0..dot_w - 1).rev() {
        perim.push((dot_h - 1, c));
    }
    // Left edge: row dot_h-2 → 1 (corners already included)
    for r in (1..dot_h - 1).rev() {
        perim.push((r, 0));
    }

    perim
}

// ── Engine ────────────────────────────────────────────────────────────────────

/// Internal animation state for one rectangle.
struct RectEngine {
    char_w: usize,
    char_h: usize,
    perimeter: Vec<(usize, usize)>,
    /// Index of the first dot in the bright arc window.
    anchor: usize,
    arc_len: usize,
    going_forward: bool,
}

impl RectEngine {
    #[allow(clippy::cast_possible_wrap)]
    fn build(char_w: usize, char_h: usize, arc_len_override: usize, spin: Spin) -> Self {
        let char_w = char_w.max(3);
        let char_h = char_h.max(2);
        let perimeter = build_perimeter(char_w, char_h);
        let n = perimeter.len();
        let arc_len = if arc_len_override > 0 {
            arc_len_override.min(n.saturating_sub(1))
        } else {
            (n / 4).max(3)
        };

        let going_forward = matches!(spin, Spin::Clockwise);
        let anchor = if going_forward {
            0
        } else {
            n.saturating_sub(arc_len)
        };

        Self {
            char_w,
            char_h,
            perimeter,
            anchor,
            arc_len,
            going_forward,
        }
    }

    /// Advance one step, bouncing when the window hits either end.
    fn walk(&mut self) {
        let n = self.perimeter.len();
        if n <= self.arc_len {
            return;
        }
        let max_anchor = n - self.arc_len;

        if self.going_forward {
            if self.anchor < max_anchor {
                self.anchor += 1;
            } else {
                self.going_forward = false;
                // Don't double-step; the next call will go backward.
            }
        } else if self.anchor > 0 {
            self.anchor -= 1;
        } else {
            self.going_forward = true;
        }
    }

    /// Render the current frame as braille [`Line`]s.
    fn render_lines(&self, arc_color: Color, dim_color: Color) -> Vec<Line<'static>> {
        let mut bright = vec![vec![0u8; self.char_w]; self.char_h];
        let mut dim = vec![vec![0u8; self.char_w]; self.char_h];

        let arc_end = self.anchor + self.arc_len;

        for (idx, &(dr, dc)) in self.perimeter.iter().enumerate() {
            let ci = dr / 4;
            let cj = dc / 2;
            if ci >= self.char_h || cj >= self.char_w {
                continue;
            }
            let bit = BRAILLE_MAP[dr % 4][dc % 2];
            if idx >= self.anchor && idx < arc_end {
                bright[ci][cj] |= 1 << bit;
            } else {
                dim[ci][cj] |= 1 << bit;
            }
        }

        (0..self.char_h)
            .map(|ri| {
                let spans: Vec<Span<'static>> = (0..self.char_w)
                    .map(|cj| {
                        let b = bright[ri][cj];
                        let d = dim[ri][cj];
                        let (byte, color) = if b != 0 {
                            (b, arc_color)
                        } else if d != 0 {
                            (d, dim_color)
                        } else {
                            (0u8, dim_color)
                        };
                        let ch =
                            char::from_u32(BRAILLE_BASE + u32::from(byte)).unwrap_or('\u{2800}');
                        Span::styled(ch.to_string(), Style::default().fg(color))
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}

// ── Public widget ─────────────────────────────────────────────────────────────

/// A Zed / Claude-style braille-dot arc that **bounces** around the perimeter
/// of a rectangle.
///
/// Unlike [`crate::RectSpinner`] and [`crate::CircleSpinner`] which rotate
/// continuously, this spinner reverses direction at each end of the perimeter,
/// producing a back-and-forth ping-pong animation.
///
/// # Layout
///
/// The rendered size is exactly `width × height` terminal character cells.
///
/// # Examples
///
/// ```no_run
/// use ratatui::style::Color;
/// use ratatui::Frame;
/// use ratatui::layout::Rect;
/// use tui_spinner::{RectangularSpinner, Spin};
///
/// fn draw(frame: &mut Frame, area: Rect, tick: u64) {
///     frame.render_widget(
///         RectangularSpinner::new(tick)
///             .width(10)
///             .height(3)
///             .spin(Spin::Clockwise)
///             .arc_color(Color::Cyan)
///             .dim_color(Color::DarkGray),
///         area,
///     );
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RectangularSpinner<'a> {
    tick: u64,
    /// Width in terminal character columns (minimum 3).
    width: usize,
    /// Height in terminal character rows (minimum 2).
    height: usize,
    /// Explicit arc length in perimeter dots (0 = auto ~¼ of perimeter).
    arc_len: usize,
    /// Starting direction before the first bounce.
    spin: Spin,
    /// Ticks held per animation step (higher = slower).
    ticks_per_step: u64,
    /// Colour of the bright arc segment.
    arc_color: Color,
    /// Colour of the dim background ring.
    dim_color: Color,
    block: Option<Block<'a>>,
    style: Style,
    alignment: Alignment,
}

impl<'a> RectangularSpinner<'a> {
    /// Creates a new [`RectangularSpinner`] with defaults:
    /// `8 × 3` characters, clockwise start, cyan arc, dark-gray ring,
    /// 1 tick per step, auto arc length.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let spinner = RectangularSpinner::new(42);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            width: 8,
            height: 3,
            arc_len: 0,
            spin: Spin::Clockwise,
            ticks_per_step: 1,
            arc_color: Color::Cyan,
            dim_color: Color::DarkGray,
            block: None,
            style: Style::default(),
            alignment: Alignment::Left,
        }
    }

    /// Sets the width in terminal character columns (minimum 3).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let wide = RectangularSpinner::new(0).width(16);
    /// ```
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.width = w.max(3);
        self
    }

    /// Sets the height in terminal character rows (minimum 2).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let tall = RectangularSpinner::new(0).height(4);
    /// ```
    #[must_use]
    pub fn height(mut self, h: usize) -> Self {
        self.height = h.max(2);
        self
    }

    /// Sets the arc length in perimeter dots (0 = auto, defaults to ~¼ perimeter).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let long_arc = RectangularSpinner::new(0).arc_len(20);
    /// ```
    #[must_use]
    pub fn arc_len(mut self, len: usize) -> Self {
        self.arc_len = len;
        self
    }

    /// Sets the starting direction before the first bounce (default: [`Spin::Clockwise`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{RectangularSpinner, Spin};
    ///
    /// let ccw = RectangularSpinner::new(0).spin(Spin::CounterClockwise);
    /// ```
    #[must_use]
    pub const fn spin(mut self, spin: Spin) -> Self {
        self.spin = spin;
        self
    }

    /// Sets how many ticks each position is held (default: 1; higher = slower).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let slow = RectangularSpinner::new(0).ticks_per_step(3);
    /// ```
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.ticks_per_step = n.max(1);
        self
    }

    /// Sets the colour of the bright arc segment (default: [`Color::Cyan`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let spinner = RectangularSpinner::new(0).arc_color(Color::LightBlue);
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
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let spinner = RectangularSpinner::new(0).dim_color(Color::Black);
    /// ```
    #[must_use]
    pub const fn dim_color(mut self, color: Color) -> Self {
        self.dim_color = color;
        self
    }

    /// Wraps the spinner in a [`Block`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::widgets::Block;
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let spinner = RectangularSpinner::new(0)
    ///     .block(Block::bordered().title("Loading…"));
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

    /// Sets the horizontal alignment of the rendered spinner (default: left).
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Returns the rendered size in terminal characters `(cols, rows)`.
    ///
    /// This always equals `(width, height)` after clamping to the minimums.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::RectangularSpinner;
    ///
    /// let (cols, rows) = RectangularSpinner::new(0).width(10).height(4).char_size();
    /// assert_eq!(cols, 10);
    /// assert_eq!(rows, 4);
    /// ```
    #[must_use]
    pub fn char_size(&self) -> (usize, usize) {
        (self.width.max(3), self.height.max(2))
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut engine = RectEngine::build(self.width, self.height, self.arc_len, self.spin);

        #[allow(clippy::cast_possible_truncation)]
        let steps = (self.tick / self.ticks_per_step) as usize;
        for _ in 0..steps {
            engine.walk();
        }

        engine.render_lines(self.arc_color, self.dim_color)
    }
}

// ── Trait impls ───────────────────────────────────────────────────────────────

impl Styled for RectangularSpinner<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for RectangularSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &RectangularSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 {
            return;
        }

        buf.set_style(area, self.style);

        let inner_area = self.block.as_ref().map_or(area, |b| {
            let inner = b.inner(area);
            Widget::render(b.clone(), area, buf);
            inner
        });

        if inner_area.area() == 0 {
            return;
        }

        let lines = self.build_lines();
        Paragraph::new(lines)
            .alignment(self.alignment)
            .render(inner_area, buf);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    // ── Perimeter geometry ────────────────────────────────────────────────────

    #[test]
    fn perimeter_length_formula() {
        // Total = 2*(2w + 4h) - 4
        for w in 3..=8usize {
            for h in 2..=6usize {
                let got = build_perimeter(w, h).len();
                let expected = 2 * (w * 2 + h * 4) - 4;
                assert_eq!(got, expected, "w={w} h={h}");
            }
        }
    }

    #[test]
    fn perimeter_no_duplicate_dots() {
        use std::collections::HashSet;
        for w in 3..=6usize {
            for h in 2..=4usize {
                let p = build_perimeter(w, h);
                let unique: HashSet<_> = p.iter().copied().collect();
                assert_eq!(unique.len(), p.len(), "duplicate dots w={w} h={h}");
            }
        }
    }

    #[test]
    fn perimeter_starts_at_top_left() {
        let p = build_perimeter(4, 2);
        assert_eq!(p[0], (0, 0), "first dot should be top-left (0,0)");
    }

    #[test]
    fn perimeter_all_dots_on_border() {
        for w in 3..=6usize {
            for h in 2..=4usize {
                let dot_w = w * 2;
                let dot_h = h * 4;
                let p = build_perimeter(w, h);
                for &(r, c) in &p {
                    let on_border = r == 0 || r == dot_h - 1 || c == 0 || c == dot_w - 1;
                    assert!(on_border, "dot ({r},{c}) is not on border w={w} h={h}");
                }
            }
        }
    }

    // ── Engine ────────────────────────────────────────────────────────────────

    #[test]
    fn engine_builds_without_panic() {
        for w in 3..=8usize {
            for h in 2..=5usize {
                for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                    let _ = RectEngine::build(w, h, 0, spin);
                }
            }
        }
    }

    #[test]
    fn engine_walk_does_not_panic() {
        let mut e = RectEngine::build(6, 3, 0, Spin::Clockwise);
        for _ in 0..500 {
            e.walk();
        }
    }

    #[test]
    fn engine_anchor_stays_in_bounds() {
        let mut e = RectEngine::build(6, 3, 0, Spin::Clockwise);
        let max_anchor = e.perimeter.len().saturating_sub(e.arc_len);
        for _ in 0..500 {
            e.walk();
            assert!(
                e.anchor <= max_anchor,
                "anchor={} max={max_anchor}",
                e.anchor
            );
        }
    }

    #[test]
    fn engine_bounces_direction() {
        let mut e = RectEngine::build(6, 3, 0, Spin::Clockwise);
        assert!(e.going_forward, "should start going forward");

        // Walk until direction reverses.
        let max_steps = e.perimeter.len() * 2;
        let mut reversed = false;
        for _ in 0..max_steps {
            e.walk();
            if !e.going_forward {
                reversed = true;
                break;
            }
        }
        assert!(reversed, "engine never reversed direction");

        // Walk until it reverses back.
        let mut re_reversed = false;
        for _ in 0..max_steps {
            e.walk();
            if e.going_forward {
                re_reversed = true;
                break;
            }
        }
        assert!(re_reversed, "engine never reversed back");
    }

    #[test]
    fn cw_and_ccw_start_differ() {
        let cw = RectEngine::build(6, 3, 0, Spin::Clockwise);
        let ccw = RectEngine::build(6, 3, 0, Spin::CounterClockwise);
        assert_ne!(
            cw.anchor, ccw.anchor,
            "CW and CCW should start at different anchors"
        );
        assert_ne!(
            cw.going_forward, ccw.going_forward,
            "CW and CCW should start with opposite directions"
        );
    }

    #[test]
    fn different_ticks_produce_different_output() {
        let lines_a = RectangularSpinner::new(0).width(8).height(3).build_lines();
        let lines_b = RectangularSpinner::new(10).width(8).height(3).build_lines();
        assert_ne!(lines_a, lines_b, "tick=0 and tick=10 should differ");
    }

    #[test]
    fn cw_and_ccw_widgets_differ_at_tick_5() {
        let cw = RectangularSpinner::new(5)
            .width(8)
            .height(3)
            .spin(Spin::Clockwise)
            .build_lines();
        let ccw = RectangularSpinner::new(5)
            .width(8)
            .height(3)
            .spin(Spin::CounterClockwise)
            .build_lines();
        assert_ne!(
            cw, ccw,
            "CW and CCW widgets should produce different output at tick 5"
        );
    }

    // ── Widget rendering ──────────────────────────────────────────────────────

    #[test]
    fn widget_renders_without_panic() {
        let backend = TestBackend::new(20, 6);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(RectangularSpinner::new(42).width(8).height(3), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn widget_zero_area_no_panic() {
        let backend = TestBackend::new(0, 0);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(RectangularSpinner::new(0), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn char_size_is_correct() {
        let s = RectangularSpinner::new(0).width(10).height(4);
        assert_eq!(s.char_size(), (10, 4));
    }

    #[test]
    fn char_size_clamps_minimum() {
        let s = RectangularSpinner::new(0).width(1).height(1);
        let (w, h) = s.char_size();
        assert!(w >= 3, "width should be at least 3");
        assert!(h >= 2, "height should be at least 2");
    }

    #[test]
    fn rendered_output_has_braille_chars() {
        let lines = RectangularSpinner::new(0).width(8).height(3).build_lines();
        assert_eq!(lines.len(), 3, "should produce 3 lines for height=3");
        for line in &lines {
            assert!(!line.spans.is_empty(), "line has no spans");
        }
    }

    #[test]
    fn builder_chain() {
        use ratatui::style::Color;
        use ratatui::widgets::Block;
        let s = RectangularSpinner::new(0)
            .width(12)
            .height(4)
            .arc_len(10)
            .spin(Spin::CounterClockwise)
            .ticks_per_step(2)
            .arc_color(Color::Blue)
            .dim_color(Color::Black)
            .block(Block::bordered())
            .alignment(Alignment::Center);
        assert_eq!(s.width, 12);
        assert_eq!(s.height, 4);
        assert_eq!(s.arc_len, 10);
        assert!(matches!(s.spin, Spin::CounterClockwise));
        assert_eq!(s.ticks_per_step, 2);
    }

    #[test]
    fn arc_len_override_respected() {
        let e = RectEngine::build(8, 3, 15, Spin::Clockwise);
        assert_eq!(e.arc_len, 15);
    }

    #[test]
    fn ticks_per_step_slows_animation() {
        let fast = RectangularSpinner::new(10)
            .width(8)
            .height(3)
            .ticks_per_step(1)
            .build_lines();
        let slow = RectangularSpinner::new(10)
            .width(8)
            .height(3)
            .ticks_per_step(5)
            .build_lines();
        assert_ne!(
            fast, slow,
            "different ticks_per_step should produce different output at tick=10"
        );
    }
}
