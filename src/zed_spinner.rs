//! Zed-style braille rotation spinner.
//!
//! Each character cell cycles through 8 braille frames where one dot is
//! missing and the gap travels **clockwise** around the cell:
//!
//! ```text
//! ⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻
//! ```
//!
//! At `width = 1` / `height = 1` this is a single animated glyph — perfect
//! as a compact status-bar indicator.  For wider or taller sizes each cell is
//! offset by `phase_step` frames from its neighbour, producing a travelling
//! diagonal wave:
//!
//! ```text
//! width = 6, phase_step = 1
//! ⣾⣷⣯⣟⡿⢿   (tick 0)
//! ⣷⣯⣟⡿⢿⣽   (tick 1)
//! ⣯⣟⡿⢿⣽⣻   (tick 2)
//! …
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use ratatui::style::Color;
//! use ratatui::Frame;
//! use ratatui::layout::Rect;
//! use tui_spinner::ZedSpinner;
//!
//! fn draw(frame: &mut Frame, area: Rect, tick: u64) {
//!     // Single-character status-bar spinner
//!     frame.render_widget(ZedSpinner::new(tick), area);
//!
//!     // Wider wave spanning a full column
//!     frame.render_widget(
//!         ZedSpinner::new(tick)
//!             .width(12)
//!             .color(Color::Cyan),
//!         area,
//!     );
//! }
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style, Styled};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

// ── Frame table ───────────────────────────────────────────────────────────────

/// 8 braille frames — full cell (`⣿`, 0xFF) with one dot missing.
///
/// The gap rotates **clockwise** through the 8 braille dot positions:
///
/// | index | byte | glyph | missing dot | position      |
/// |-------|------|-------|-------------|---------------|
/// |   0   | 0xFE | `⣾`  | dot 1       | top-left      |
/// |   1   | 0xF7 | `⣷`  | dot 4       | top-right     |
/// |   2   | 0xEF | `⣯`  | dot 5       | mid-right     |
/// |   3   | 0xDF | `⣟`  | dot 6       | bot-mid-right |
/// |   4   | 0x7F | `⡿`  | dot 8       | bot-right     |
/// |   5   | 0xBF | `⢿`  | dot 7       | bot-left      |
/// |   6   | 0xFD | `⣽`  | dot 2       | mid-left      |
/// |   7   | 0xFB | `⣻`  | dot 3       | bot-mid-left  |
const FRAMES: [u8; 8] = [
    0xFE, // ⣾  dot 1 missing  (top-left)
    0xF7, // ⣷  dot 4 missing  (top-right)
    0xEF, // ⣯  dot 5 missing  (mid-right)
    0xDF, // ⣟  dot 6 missing  (bot-mid-right)
    0x7F, // ⡿  dot 8 missing  (bot-right)
    0xBF, // ⢿  dot 7 missing  (bot-left)
    0xFD, // ⣽  dot 2 missing  (mid-left)
    0xFB, // ⣻  dot 3 missing  (bot-mid-left)
];

// ── Public widget ─────────────────────────────────────────────────────────────

/// A Zed-style braille rotation spinner.
///
/// At its smallest (`1 × 1`) a single character cycles through eight frames
/// where one braille dot is missing and the gap rotates clockwise — giving the
/// characteristic "spinning hole" look of Zed's activity indicator.
///
/// Scaling up via [`width`](ZedSpinner::width) / [`height`](ZedSpinner::height)
/// adds a configurable per-cell phase offset ([`phase_step`](ZedSpinner::phase_step))
/// so adjacent characters are staggered in time, producing a smooth diagonal
/// wave across the spinner block.
///
/// # Default values
///
/// | Field           | Default              |
/// |-----------------|----------------------|
/// | `width`         | `1`                  |
/// | `height`        | `1`                  |
/// | `color`         | [`Color::Cyan`]      |
/// | `ticks_per_step`| `1`                  |
/// | `phase_step`    | `1`                  |
///
/// # Examples
///
/// ```
/// use tui_spinner::ZedSpinner;
///
/// // Minimal 1×1 spinner
/// let s = ZedSpinner::new(42);
///
/// // 8-wide wave
/// let wave = ZedSpinner::new(42).width(8).phase_step(1);
/// ```
#[derive(Debug, Clone)]
pub struct ZedSpinner<'a> {
    tick: u64,
    /// Width in character columns (default 1).
    width: usize,
    /// Height in character rows (default 1).
    height: usize,
    /// Colour of each spinner glyph (default [`Color::Cyan`]).
    color: Color,
    /// Ticks held per animation frame (default 1; higher = slower).
    ticks_per_step: u64,
    /// Frame offset added to each successive cell (default 1).
    ///
    /// `0` → all cells are synchronised.
    /// `1` → each cell is 1 frame ahead of its left/upper neighbour.
    /// `4` → cells 4 frames apart have opposite phase (⣾ vs ⡿).
    phase_step: u8,
    block: Option<Block<'a>>,
    style: Style,
    alignment: Alignment,
}

impl<'a> ZedSpinner<'a> {
    /// Creates a new [`ZedSpinner`] with default settings: `1 × 1`, cyan,
    /// 1 tick per frame, phase step 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::ZedSpinner;
    ///
    /// let s = ZedSpinner::new(0);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            width: 1,
            height: 1,
            color: Color::Cyan,
            ticks_per_step: 1,
            phase_step: 1,
            block: None,
            style: Style::default(),
            alignment: Alignment::Left,
        }
    }

    /// Sets the width in character columns (default 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::ZedSpinner;
    ///
    /// let wide = ZedSpinner::new(0).width(6);
    /// ```
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.width = w.max(1);
        self
    }

    /// Sets the height in character rows (default 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::ZedSpinner;
    ///
    /// let tall = ZedSpinner::new(0).height(3);
    /// ```
    #[must_use]
    pub fn height(mut self, h: usize) -> Self {
        self.height = h.max(1);
        self
    }

    /// Sets the spinner colour (default [`Color::Cyan`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::ZedSpinner;
    ///
    /// let s = ZedSpinner::new(0).color(Color::White);
    /// ```
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets how many ticks each frame is held (default 1; higher = slower).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::ZedSpinner;
    ///
    /// let slow = ZedSpinner::new(0).ticks_per_step(4);
    /// ```
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.ticks_per_step = n.max(1);
        self
    }

    /// Sets the frame offset between adjacent cells (default 1).
    ///
    /// | value | effect                                              |
    /// |-------|-----------------------------------------------------|
    /// | `0`   | All cells synchronised — a uniform pulsing block    |
    /// | `1`   | Smooth diagonal wave (default)                      |
    /// | `2`   | Faster wave                                         |
    /// | `4`   | Cells in anti-phase: neighbouring cells spin opposite|
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::ZedSpinner;
    ///
    /// let sync  = ZedSpinner::new(0).width(4).phase_step(0);
    /// let wave  = ZedSpinner::new(0).width(4).phase_step(1);
    /// let anti  = ZedSpinner::new(0).width(4).phase_step(4);
    /// ```
    #[must_use]
    pub const fn phase_step(mut self, step: u8) -> Self {
        self.phase_step = step;
        self
    }

    /// Wraps the spinner in a [`Block`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::widgets::Block;
    /// use tui_spinner::ZedSpinner;
    ///
    /// let s = ZedSpinner::new(0).block(Block::bordered().title("Indexing…"));
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

    /// Returns the rendered size in character cells `(cols, rows)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::ZedSpinner;
    ///
    /// assert_eq!(ZedSpinner::new(0).width(5).height(2).char_size(), (5, 2));
    /// ```
    #[must_use]
    pub fn char_size(&self) -> (usize, usize) {
        (self.width.max(1), self.height.max(1))
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        #[allow(clippy::cast_possible_truncation)]
        let base_frame = (self.tick / self.ticks_per_step) as u8;

        (0..self.height)
            .map(|r| {
                let spans: Vec<Span<'static>> = (0..self.width)
                    .map(|c| {
                        // Linear cell index, then apply phase offset.
                        #[allow(clippy::cast_possible_truncation)]
                        let cell_idx = (r * self.width + c) as u8;
                        let phase = cell_idx.wrapping_mul(self.phase_step);
                        let byte = FRAMES[((base_frame.wrapping_add(phase)) % 8) as usize];
                        let ch = char::from_u32(0x2800 + u32::from(byte)).unwrap_or('⣾');
                        Span::styled(ch.to_string(), Style::default().fg(self.color))
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}

// ── Trait impls ───────────────────────────────────────────────────────────────

impl Styled for ZedSpinner<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for ZedSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &ZedSpinner<'_> {
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

    // ── Frame table ───────────────────────────────────────────────────────────

    #[test]
    fn all_frames_are_distinct() {
        let mut seen = std::collections::HashSet::new();
        for &b in &FRAMES {
            assert!(seen.insert(b), "duplicate frame byte 0x{b:02X}");
        }
    }

    #[test]
    fn all_frames_are_full_minus_one_dot() {
        for &b in &FRAMES {
            // Each frame must have exactly 7 of 8 bits set.
            assert_eq!(
                b.count_ones(),
                7,
                "frame 0x{b:02X} should have 7 bits set (one dot missing)"
            );
        }
    }

    #[test]
    fn frames_cover_all_eight_bit_positions() {
        // Each frame has a different bit cleared.
        let cleared_bits: Vec<u8> = FRAMES.iter().map(|&b| (!b) & 0xFF).collect();
        for (i, &mask) in cleared_bits.iter().enumerate() {
            assert_eq!(
                mask.count_ones(),
                1,
                "frame {i}: cleared mask 0x{mask:02X} should have exactly 1 bit"
            );
        }
        // All 8 bit positions are represented.
        let combined: u8 = cleared_bits.iter().fold(0u8, |acc, &m| acc | m);
        assert_eq!(combined, 0xFF, "all 8 dot positions must be covered");
    }

    // ── Animation ─────────────────────────────────────────────────────────────

    #[test]
    fn advances_each_tick() {
        let f0 = ZedSpinner::new(0).build_lines();
        let f1 = ZedSpinner::new(1).build_lines();
        assert_ne!(f0, f1, "consecutive ticks should produce different frames");
    }

    #[test]
    fn wraps_after_eight_steps() {
        let f0 = ZedSpinner::new(0).build_lines();
        let f8 = ZedSpinner::new(8).build_lines();
        assert_eq!(f0, f8, "should wrap back to frame 0 after 8 ticks");
    }

    #[test]
    fn ticks_per_step_slows_animation() {
        // With ticks_per_step=4, tick 0 and tick 3 should show the same frame.
        let a = ZedSpinner::new(0).ticks_per_step(4).build_lines();
        let b = ZedSpinner::new(3).ticks_per_step(4).build_lines();
        assert_eq!(
            a, b,
            "ticks 0–3 should all be frame 0 when ticks_per_step=4"
        );

        // Tick 4 advances to the next frame.
        let c = ZedSpinner::new(4).ticks_per_step(4).build_lines();
        assert_ne!(a, c, "tick 4 should advance to frame 1");
    }

    // ── Phase wave ────────────────────────────────────────────────────────────

    #[test]
    fn phase_step_zero_all_cells_same() {
        let lines = ZedSpinner::new(0).width(4).phase_step(0).build_lines();
        let spans = &lines[0].spans;
        let first = &spans[0].content;
        for s in spans.iter().skip(1) {
            assert_eq!(&s.content, first, "phase_step=0 → all cells identical");
        }
    }

    #[test]
    fn phase_step_one_cells_differ() {
        let lines = ZedSpinner::new(0).width(4).phase_step(1).build_lines();
        let spans = &lines[0].spans;
        // With phase_step=1 and 4 cells, adjacent cells must differ.
        for pair in spans.windows(2) {
            assert_ne!(
                pair[0].content, pair[1].content,
                "adjacent cells should differ with phase_step=1"
            );
        }
    }

    #[test]
    fn phase_step_eight_wraps_to_same() {
        // phase_step=8 ≡ phase_step=0 mod 8.
        let base = ZedSpinner::new(0).width(3).phase_step(0).build_lines();
        let wrap = ZedSpinner::new(0).width(3).phase_step(8).build_lines();
        assert_eq!(base, wrap, "phase_step=8 should behave like phase_step=0");
    }

    // ── Size ──────────────────────────────────────────────────────────────────

    #[test]
    fn output_dimensions_match_width_height() {
        for w in 1..=5usize {
            for h in 1..=3usize {
                let lines = ZedSpinner::new(0).width(w).height(h).build_lines();
                assert_eq!(lines.len(), h, "height={h}");
                for (i, line) in lines.iter().enumerate() {
                    assert_eq!(line.spans.len(), w, "row {i}: width={w}");
                }
            }
        }
    }

    #[test]
    fn char_size_returns_width_height() {
        let s = ZedSpinner::new(0).width(4).height(2);
        assert_eq!(s.char_size(), (4, 2));
    }

    #[test]
    fn char_size_clamps_to_one() {
        let s = ZedSpinner::new(0).width(0).height(0);
        let (w, h) = s.char_size();
        assert!(w >= 1);
        assert!(h >= 1);
    }

    // ── Widget rendering ──────────────────────────────────────────────────────

    #[test]
    fn widget_renders_without_panic() {
        let backend = TestBackend::new(10, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(ZedSpinner::new(42).width(3).height(2), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn widget_single_char_renders() {
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(ZedSpinner::new(0), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn widget_zero_area_no_panic() {
        let backend = TestBackend::new(0, 0);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(ZedSpinner::new(0), frame.area());
            })
            .unwrap();
    }

    // ── Builder chain ─────────────────────────────────────────────────────────

    #[test]
    fn builder_chain() {
        use ratatui::widgets::Block;
        let s = ZedSpinner::new(99)
            .width(6)
            .height(2)
            .color(Color::Green)
            .ticks_per_step(3)
            .phase_step(2)
            .block(Block::bordered())
            .alignment(Alignment::Center);
        assert_eq!(s.width, 6);
        assert_eq!(s.height, 2);
        assert_eq!(s.color, Color::Green);
        assert_eq!(s.ticks_per_step, 3);
        assert_eq!(s.phase_step, 2);
    }

    #[test]
    fn output_contains_only_valid_braille() {
        for tick in 0..8u64 {
            let lines = ZedSpinner::new(tick).width(4).height(2).build_lines();
            for line in &lines {
                for span in &line.spans {
                    let ch = span.content.chars().next().unwrap();
                    assert!(
                        ('\u{2800}'..='\u{28FF}').contains(&ch),
                        "character U+{:04X} is not a braille glyph",
                        ch as u32
                    );
                }
            }
        }
    }
}
