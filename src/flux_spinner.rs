//! Braille rotation spinner.
//!
//! Each character cell cycles through 8 braille frames where one dot is
//! missing and the gap travels around the cell.  Direction is controlled by
//! [`Spin`]:
//!
//! ```text
//! Clockwise        ⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻  →  ⣾ …
//! CounterClockwise ⣾ ⣻ ⣽ ⢿ ⡿ ⣟ ⣯ ⣷  →  ⣾ …
//! ```
//!
//! At `width = 1` / `height = 1` this is a single animated glyph — perfect
//! as a compact status-bar indicator.  For wider or taller sizes each cell is
//! offset by [`phase_step`](FluxSpinner::phase_step) frames from its neighbour,
//! producing a travelling diagonal wave:
//!
//! ```text
//! width = 6, phase_step = 1, Clockwise
//! ⣾⣷⣯⣟⡿⢿   (tick 0)
//! ⣷⣯⣟⡿⢿⣽   (tick 1)
//! ⣯⣟⡿⢿⣽⣻   (tick 2)
//! …
//! ```
//!
//! With [`Spin::CounterClockwise`] the wave travels in the opposite direction.
//!
//! ## Usage
//!
//! ```no_run
//! use ratatui::style::Color;
//! use ratatui::Frame;
//! use ratatui::layout::Rect;
//! use tui_spinner::{FluxSpinner, Spin};
//!
//! fn draw(frame: &mut Frame, area: Rect, tick: u64) {
//!     // Single-character status-bar spinner (clockwise, default)
//!     frame.render_widget(FluxSpinner::new(tick), area);
//!
//!     // Counter-clockwise wave spanning a full column
//!     frame.render_widget(
//!         FluxSpinner::new(tick)
//!             .width(12)
//!             .spin(Spin::CounterClockwise)
//!             .color(Color::Cyan),
//!         area,
//!     );
//! }
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Widget};

use crate::Spin;

// ── Frame presets ─────────────────────────────────────────────────────────────

/// Built-in frame sequences for [`FluxSpinner`].
///
/// Pass any preset (or a custom `&'static [char]` slice) to
/// [`FluxSpinner::frames`] to change the animation glyphs.
///
/// | Preset     | Glyphs                        | Frames | Description                         |
/// |------------|-------------------------------|--------|-------------------------------------|
/// | `BRAILLE`  | `⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻`     | 8      | Full cell, one dot missing (default)|
/// | `ORBIT`    | `⠁ ⠈ ⠐ ⠠ ⢀ ⡀ ⠄ ⠂`     | 8      | Single dot orbiting (inverse)       |
/// | `CLASSIC`  | `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏` | 10     | Classic braille spinner             |
/// | `LINE`     | `│ ╱ ─ ╲`                 | 4      | Rotating line                       |
/// | `BLOCK`    | `▖ ▘ ▝ ▗`                 | 4      | Quarter-block rotation              |
/// | `ARC`      | `◜ ◝ ◞ ◟`                 | 4      | Quarter-arc rotation                |
/// | `CLOCK`     | `◷ ◶ ◵ ◴`                     | 4      | Quarter-circle pie slice      |
/// | `MOON`      | `◓ ◑ ◒ ◐`                     | 4      | Half-circle moon phase        |
/// | `TRIANGLES` | `▲ ▶ ▼ ◀`                     | 4      | Filled triangle four dirs     |
/// | `PULSE`     | `⣀ ⣤ ⣶ ⣾ ⣿ ⣾ ⣶ ⣤`         | 8      | Braille fill pulse            |
/// | `BOUNCE`    | `⠉ ⠒ ⣀ ⠒`                    | 4      | Braille row bouncing top→mid→bottom |
/// | `HALF`      | `▀ ▐ ▄ ▌`                     | 4      | Half-block rotating clockwise |
/// | `SQUARE`    | `◰ ◳ ◲ ◱`                     | 4      | White square, one filled quadrant   |
/// | `DICE`      | `⚀ ⚁ ⚂ ⚃ ⚄ ⚅`               | 6      | Dice faces one to six         |
/// | `BAR`       | `▁ ▂ ▃ ▄ ▅ ▆ ▇ █`             | 8      | Sub-block growing bar         |
/// | `CORNERS`     | `┌ ┐ ┘ └`                     | 4      | Box-drawing corners rotate    |
/// | `CIRCLE_FILL` | `○ ◔ ◑ ◕ ●`                   | 5      | Circle filling clockwise      |
/// | `PISTON`      | `▁ ▃ ▅ ▇ █ ▇ ▅ ▃`             | 8      | Bouncing bar (repeats)        |
/// | `STAR`        | `✶ ✷ ✸ ✹`                     | 4      | Braille-asterisk star ramp    |
/// | `PAIR`        | `⠉ ⠘ ⠰ ⢠ ⣀ ⡄ ⠆ ⠃`         | 8      | Two dots rotating together    |
/// | `DIAMOND`     | `◇ ◈ ◆ ◈`                     | 4      | Diamond pulse (repeats)       |
///
/// # Examples
///
/// ```
/// use tui_spinner::{FluxSpinner, FluxFrames};
///
/// let braille = FluxSpinner::new(0);  // BRAILLE is the default
/// let orbit   = FluxSpinner::new(0).frames(FluxFrames::ORBIT);
/// let line    = FluxSpinner::new(0).frames(FluxFrames::LINE);
/// let custom  = FluxSpinner::new(0).frames(&['a', 'b', 'c', 'd']);
/// ```
pub struct FluxFrames;

impl FluxFrames {
    /// Full braille cell with one dot missing — the gap rotates clockwise.
    ///
    /// `⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻` — **default**.
    pub const BRAILLE: &'static [char] = &['⣾', '⣷', '⣯', '⣟', '⡿', '⢿', '⣽', '⣻'];

    /// Single braille dot orbiting clockwise — visual complement of `BRAILLE`.
    ///
    /// `⠁ ⠈ ⠐ ⠠ ⢀ ⡀ ⠄ ⠂`
    pub const ORBIT: &'static [char] = &['⠁', '⠈', '⠐', '⠠', '⢀', '⡀', '⠄', '⠂'];

    /// Classic 10-frame braille spinner.
    ///
    /// `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏`
    pub const CLASSIC: &'static [char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

    /// Rotating line — 4 frames.
    ///
    /// `│ ╱ ─ ╲`
    pub const LINE: &'static [char] = &['│', '╱', '─', '╲'];

    /// Quarter-block rotation — 4 frames.
    ///
    /// `▖ ▘ ▝ ▗`
    pub const BLOCK: &'static [char] = &['▖', '▘', '▝', '▗'];

    /// Quarter-arc rotation — 4 frames.
    ///
    /// `◜ ◝ ◞ ◟`
    pub const ARC: &'static [char] = &['◜', '◝', '◞', '◟'];

    /// Quarter-circle pie slice rotating through four positions.
    ///
    /// `◷ ◶ ◵ ◴`
    pub const CLOCK: &'static [char] = &['◷', '◶', '◵', '◴'];

    /// Half-circle (moon phase) rotating through four positions.
    ///
    /// `◓ ◑ ◒ ◐`
    pub const MOON: &'static [char] = &['◓', '◑', '◒', '◐'];

    /// Filled triangle pointing in four directions.
    ///
    /// `▲ ▶ ▼ ◀`
    pub const TRIANGLES: &'static [char] = &['▲', '▶', '▼', '◀'];

    /// Braille fill pulsing from a thin baseline up to full density and back.
    ///
    /// `⣀ ⣤ ⣶ ⣾ ⣿ ⣾ ⣶ ⣤`
    pub const PULSE: &'static [char] = &['⣀', '⣤', '⣶', '⣾', '⣿', '⣾', '⣶', '⣤'];

    /// Single braille row bouncing top → middle → bottom → middle.
    ///
    /// `⠉ ⠒ ⣀ ⠒`
    ///
    /// `⠒` is intentionally repeated — that is the bounce return step.
    pub const BOUNCE: &'static [char] = &['⠉', '⠒', '⣀', '⠒'];

    /// Half-block rotating clockwise through four positions.
    ///
    /// `▀ ▐ ▄ ▌`
    pub const HALF: &'static [char] = &['▀', '▐', '▄', '▌'];

    /// White square with one filled quadrant rotating clockwise.
    ///
    /// `◰ ◳ ◲ ◱`
    pub const SQUARE: &'static [char] = &['◰', '◳', '◲', '◱'];

    /// Dice faces cycling from one to six.
    ///
    /// `⚀ ⚁ ⚂ ⚃ ⚄ ⚅`
    pub const DICE: &'static [char] = &['⚀', '⚁', '⚂', '⚃', '⚄', '⚅'];

    /// Eight sub-block glyphs growing from one-eighth to full height.
    ///
    /// `▁ ▂ ▃ ▄ ▅ ▆ ▇ █`
    ///
    /// Clockwise runs the bar upward; counter-clockwise shrinks it back down.
    pub const BAR: &'static [char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    /// Box-drawing corners rotating clockwise.
    ///
    /// `┌ ┐ ┘ └`
    pub const CORNERS: &'static [char] = &['┌', '┐', '┘', '└'];

    /// Circle gradually filling clockwise through five stages.
    ///
    /// `○ ◔ ◑ ◕ ●`
    pub const CIRCLE_FILL: &'static [char] = &['○', '◔', '◑', '◕', '●'];

    /// Bar that bounces from one-eighth height to full and back.
    ///
    /// `▁ ▃ ▅ ▇ █ ▇ ▅ ▃`
    ///
    /// `▃`, `▅`, `▇` are intentionally repeated — that is the bounce return.
    pub const PISTON: &'static [char] = &['▁', '▃', '▅', '▇', '█', '▇', '▅', '▃'];

    /// Four braille-asterisk star glyphs increasing in density.
    ///
    /// `✶ ✷ ✸ ✹`
    pub const STAR: &'static [char] = &['✶', '✷', '✸', '✹'];

    /// Two adjacent braille dots rotating clockwise around the cell.
    ///
    /// `⠉ ⠘ ⠰ ⢠ ⣀ ⡄ ⠆ ⠃`
    pub const PAIR: &'static [char] = &['⠉', '⠘', '⠰', '⢠', '⣀', '⡄', '⠆', '⠃'];

    /// Diamond pulsing between hollow, dotted, and solid.
    ///
    /// `◇ ◈ ◆ ◈`
    ///
    /// `◈` is intentionally repeated — that is the pulse return step.
    pub const DIAMOND: &'static [char] = &['◇', '◈', '◆', '◈'];
}

// ── Public widget ─────────────────────────────────────────────────────────────

/// A compact braille rotation spinner.
///
/// Each character cell shows a full 8-dot braille glyph (`⣿`) with one dot
/// missing; the gap rotates through all eight positions every 8 steps,
/// creating an animated "spinning hole" effect.
///
/// Direction is controlled by [`Spin`]:
/// - [`Spin::Clockwise`] (default) — gap moves clockwise: `⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻`
/// - [`Spin::CounterClockwise`]    — gap moves counter-clockwise: `⣾ ⣻ ⣽ ⢿ ⡿ ⣟ ⣯ ⣷`
///
/// Scaling up via [`width`](FluxSpinner::width) / [`height`](FluxSpinner::height)
/// adds a configurable per-cell phase offset
/// ([`phase_step`](FluxSpinner::phase_step)) so adjacent characters are
/// staggered in time, producing a smooth diagonal wave across the spinner
/// block.  The wave direction follows the spin direction.
///
/// # Default values
///
/// | Field            | Default                     |
/// |------------------|-----------------------------|
/// | `width`          | `1`                         |
/// | `height`         | `1`                         |
/// | `spin`           | [`Spin::Clockwise`]         |
/// | `color`          | [`Color::Cyan`]             |
/// | `ticks_per_step` | `1`                         |
/// | `phase_step`     | `1`                         |
/// | `frames`         | [`FluxFrames::BRAILLE`]     |
///
/// # Examples
///
/// ```
/// use tui_spinner::{FluxFrames, FluxSpinner, Spin};
///
/// // Minimal 1×1 clockwise spinner
/// let s = FluxSpinner::new(42);
///
/// // 8-wide counter-clockwise wave
/// let wave = FluxSpinner::new(42)
///     .width(8)
///     .spin(Spin::CounterClockwise)
///     .phase_step(1);
///
/// // Custom frame sequence
/// let line = FluxSpinner::new(42).frames(FluxFrames::LINE);
/// ```
///
/// ## Embedding in another widget
///
/// Because the spinner implements `Into<Text>`, it can be dropped straight
/// into any widget that accepts text content — such as a table
/// [`Cell`](ratatui::widgets::Cell) or a
/// [`Paragraph`](ratatui::widgets::Paragraph) — with no special method:
///
/// ```
/// use ratatui::style::Color;
/// use ratatui::widgets::{Cell, Paragraph};
/// use tui_spinner::{FluxSpinner, Spin};
///
/// let spinner = FluxSpinner::new(3)
///     .width(12)
///     .spin(Spin::CounterClockwise)
///     .color(Color::Cyan);
///
/// // By reference …
/// let _cell = Cell::from(&spinner);
/// // … or by value, or straight into a Paragraph
/// let _para = Paragraph::new(FluxSpinner::new(0).width(8));
/// ```
///
/// When you need to combine the spinner rows with other text in the same
/// cell, use [`to_lines`](FluxSpinner::to_lines):
///
/// ```
/// use ratatui::text::Line;
/// use ratatui::widgets::Cell;
/// use tui_spinner::FluxSpinner;
///
/// let spinner = FluxSpinner::new(3).width(12);
/// let mut lines: Vec<Line> = vec![Line::from("The cell content")];
/// lines.extend(spinner.to_lines());
/// let _cell = Cell::from(lines);
/// ```
#[derive(Debug, Clone)]
pub struct FluxSpinner<'a> {
    tick: u64,
    /// Width in character columns (default 1).
    width: usize,
    /// Height in character rows (default 1).
    height: usize,
    /// Rotation direction (default [`Spin::Clockwise`]).
    spin: Spin,
    /// Colour of each spinner glyph (default [`Color::Cyan`]).
    color: Color,
    /// Ticks held per animation frame (default 1; higher = slower).
    ticks_per_step: u64,
    /// Frame offset added to each successive cell (default 1).
    ///
    /// `0` → all cells are synchronised (uniform pulse).
    /// `1` → each cell is 1 frame ahead of its left/upper neighbour (smooth
    ///        wave in the spin direction).
    /// `4` → cells 4 frames apart have opposite phase (`⣾` vs `⡿`).
    phase_step: u8,
    /// The frame sequence to animate through (default [`FluxFrames::BRAILLE`]).
    frames: &'static [char],
    block: Option<Block<'a>>,
    style: Style,
    alignment: Alignment,
}

impl<'a> FluxSpinner<'a> {
    /// Creates a new [`FluxSpinner`] with default settings: `1 × 1`,
    /// clockwise, cyan, 1 tick per frame, phase step 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::FluxSpinner;
    ///
    /// let s = FluxSpinner::new(0);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            width: 1,
            height: 1,
            spin: Spin::Clockwise,
            color: Color::Cyan,
            ticks_per_step: 1,
            phase_step: 1,
            frames: FluxFrames::BRAILLE,
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
    /// use tui_spinner::FluxSpinner;
    ///
    /// let wide = FluxSpinner::new(0).width(6);
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
    /// use tui_spinner::FluxSpinner;
    ///
    /// let tall = FluxSpinner::new(0).height(3);
    /// ```
    #[must_use]
    pub fn height(mut self, h: usize) -> Self {
        self.height = h.max(1);
        self
    }

    /// Sets the rotation direction (default [`Spin::Clockwise`]).
    ///
    /// Also reverses the phase-wave direction for multi-cell spinners.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{FluxSpinner, Spin};
    ///
    /// let cw  = FluxSpinner::new(0).spin(Spin::Clockwise);
    /// let ccw = FluxSpinner::new(0).spin(Spin::CounterClockwise);
    /// ```
    #[must_use]
    pub const fn spin(mut self, spin: Spin) -> Self {
        self.spin = spin;
        self
    }

    /// Sets the spinner colour (default [`Color::Cyan`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::FluxSpinner;
    ///
    /// let s = FluxSpinner::new(0).color(Color::White);
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
    /// use tui_spinner::FluxSpinner;
    ///
    /// let slow = FluxSpinner::new(0).ticks_per_step(4);
    /// ```
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.ticks_per_step = n.max(1);
        self
    }

    /// Sets the frame offset between adjacent cells (default 1).
    ///
    /// | value | effect                                               |
    /// |-------|------------------------------------------------------|
    /// | `0`   | All cells synchronised — a uniform pulsing block     |
    /// | `1`   | Smooth diagonal wave (default)                       |
    /// | `2`   | Faster / wider wave                                  |
    /// | `4`   | Anti-phase: neighbouring cells spin opposite (`⣾`/`⡿`)|
    ///
    /// The wave travels in the [`spin`](FluxSpinner::spin) direction.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::FluxSpinner;
    ///
    /// let sync = FluxSpinner::new(0).width(4).phase_step(0);
    /// let wave = FluxSpinner::new(0).width(4).phase_step(1);
    /// let anti = FluxSpinner::new(0).width(4).phase_step(4);
    /// ```
    #[must_use]
    pub const fn phase_step(mut self, step: u8) -> Self {
        self.phase_step = step;
        self
    }

    /// Sets the frame sequence (default [`FluxFrames::BRAILLE`]).
    ///
    /// Use one of the [`FluxFrames`] presets or supply any
    /// `&'static [char]` slice for a fully custom animation.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{FluxSpinner, FluxFrames};
    ///
    /// let orbit  = FluxSpinner::new(0).frames(FluxFrames::ORBIT);
    /// let line   = FluxSpinner::new(0).frames(FluxFrames::LINE);
    /// let custom = FluxSpinner::new(0).frames(&['◐', '◓', '◑', '◒']);
    /// ```
    #[must_use]
    pub fn frames(mut self, frames: &'static [char]) -> Self {
        self.frames = frames;
        self
    }

    /// Wraps the spinner in a [`Block`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::widgets::Block;
    /// use tui_spinner::FluxSpinner;
    ///
    /// let s = FluxSpinner::new(0).block(Block::bordered().title("Indexing…"));
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
    /// use tui_spinner::FluxSpinner;
    ///
    /// assert_eq!(FluxSpinner::new(0).width(5).height(2).char_size(), (5, 2));
    /// ```
    #[must_use]
    pub fn char_size(&self) -> (usize, usize) {
        (self.width.max(1), self.height.max(1))
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let n = self.frames.len();
        if n == 0 {
            return vec![];
        }

        // Use usize throughout — avoids cast truncation on large tick values.
        #[allow(clippy::cast_possible_truncation)]
        let base = (self.tick / self.ticks_per_step) as usize;
        let ccw = matches!(self.spin, Spin::CounterClockwise);

        (0..self.height)
            .map(|r| {
                let spans: Vec<Span<'static>> = (0..self.width)
                    .map(|c| {
                        let cell_idx = r * self.width + c;
                        let phase = cell_idx * usize::from(self.phase_step);
                        let raw = base.wrapping_add(phase);

                        // CW:  advance through frames.
                        // CCW: retreat through frames (reverses wave too).
                        let frame_idx = if ccw { (n - raw % n) % n } else { raw % n };

                        let ch = self.frames[frame_idx];
                        Span::styled(ch.to_string(), Style::default().fg(self.color))
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}

// ── Trait impls ───────────────────────────────────────────────────────────────

impl_styled_for!(FluxSpinner<'_>);

impl_to_text!(FluxSpinner<'_>, build_lines);

impl_widget_via_ref!(FluxSpinner<'_>);

impl Widget for &FluxSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_spinner_body!(self, area, buf, self.build_lines());
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    // ── Frame table ───────────────────────────────────────────────────────────

    #[test]
    fn braille_preset_has_eight_frames() {
        assert_eq!(FluxFrames::BRAILLE.len(), 8);
    }

    #[test]
    fn all_presets_non_empty() {
        assert!(!FluxFrames::BRAILLE.is_empty());
        assert!(!FluxFrames::ORBIT.is_empty());
        assert!(!FluxFrames::CLASSIC.is_empty());
        assert!(!FluxFrames::LINE.is_empty());
        assert!(!FluxFrames::BLOCK.is_empty());
        assert!(!FluxFrames::ARC.is_empty());
        assert!(!FluxFrames::BOUNCE.is_empty());
        assert!(!FluxFrames::HALF.is_empty());
        assert!(!FluxFrames::SQUARE.is_empty());
        assert!(!FluxFrames::DICE.is_empty());
        assert!(!FluxFrames::CLOCK.is_empty());
        assert!(!FluxFrames::MOON.is_empty());
        assert!(!FluxFrames::TRIANGLES.is_empty());
        assert!(!FluxFrames::PULSE.is_empty());
        assert!(!FluxFrames::BAR.is_empty());
        assert!(!FluxFrames::CORNERS.is_empty());
        assert!(!FluxFrames::CIRCLE_FILL.is_empty());
        assert!(!FluxFrames::PISTON.is_empty());
        assert!(!FluxFrames::STAR.is_empty());
        assert!(!FluxFrames::PAIR.is_empty());
        assert!(!FluxFrames::DIAMOND.is_empty());
    }

    #[test]
    fn all_presets_have_distinct_chars_within_set() {
        // PULSE deliberately repeats chars (it pulses, not rotates).
        // BOUNCE repeats ⠒ by design (return step).
        // PISTON and DIAMOND repeat chars by design (bounce/pulse return steps).
        for (name, preset) in [
            ("BRAILLE", FluxFrames::BRAILLE),
            ("ORBIT", FluxFrames::ORBIT),
            ("CLASSIC", FluxFrames::CLASSIC),
            ("LINE", FluxFrames::LINE),
            ("BLOCK", FluxFrames::BLOCK),
            ("ARC", FluxFrames::ARC),
            ("CLOCK", FluxFrames::CLOCK),
            ("MOON", FluxFrames::MOON),
            ("TRIANGLES", FluxFrames::TRIANGLES),
            ("HALF", FluxFrames::HALF),
            ("SQUARE", FluxFrames::SQUARE),
            ("DICE", FluxFrames::DICE),
            ("BAR", FluxFrames::BAR),
            ("CORNERS", FluxFrames::CORNERS),
            ("CIRCLE_FILL", FluxFrames::CIRCLE_FILL),
            ("STAR", FluxFrames::STAR),
            ("PAIR", FluxFrames::PAIR),
        ] {
            let unique: std::collections::HashSet<char> = preset.iter().copied().collect();
            assert_eq!(unique.len(), preset.len(), "{name} has duplicate chars");
        }
    }

    // ── Clockwise animation ───────────────────────────────────────────────────

    #[test]
    fn cw_advances_each_tick() {
        let f0 = FluxSpinner::new(0).spin(Spin::Clockwise).build_lines();
        let f1 = FluxSpinner::new(1).spin(Spin::Clockwise).build_lines();
        assert_ne!(f0, f1, "consecutive ticks should produce different frames");
    }

    #[test]
    fn cw_wraps_after_eight_steps() {
        let f0 = FluxSpinner::new(0).spin(Spin::Clockwise).build_lines();
        let f8 = FluxSpinner::new(8).spin(Spin::Clockwise).build_lines();
        assert_eq!(f0, f8, "should wrap back to frame 0 after 8 ticks");
    }

    #[test]
    fn ticks_per_step_slows_animation() {
        let a = FluxSpinner::new(0).ticks_per_step(4).build_lines();
        let b = FluxSpinner::new(3).ticks_per_step(4).build_lines();
        assert_eq!(
            a, b,
            "ticks 0–3 should all be frame 0 when ticks_per_step=4"
        );

        let c = FluxSpinner::new(4).ticks_per_step(4).build_lines();
        assert_ne!(a, c, "tick 4 should advance to frame 1");
    }

    // ── Direction ─────────────────────────────────────────────────────────────

    #[test]
    fn cw_and_ccw_differ_at_same_tick() {
        let cw = FluxSpinner::new(1).spin(Spin::Clockwise).build_lines();
        let ccw = FluxSpinner::new(1)
            .spin(Spin::CounterClockwise)
            .build_lines();
        assert_ne!(
            cw, ccw,
            "CW and CCW should produce different frames at tick 1"
        );
    }

    #[test]
    fn cw_and_ccw_agree_at_tick_zero() {
        // Frame index 0 for CW: (8 - 0) % 8 == 0 for CCW — both start at FRAMES[0].
        let cw = FluxSpinner::new(0).spin(Spin::Clockwise).build_lines();
        let ccw = FluxSpinner::new(0)
            .spin(Spin::CounterClockwise)
            .build_lines();
        assert_eq!(cw, ccw, "both directions share frame 0 at tick 0");
    }

    #[test]
    fn ccw_is_reverse_of_cw() {
        // CW tick 1 == CCW tick 7, because (8 - 1) % 8 == 7 and 1 % 8 == 1
        // are symmetric: CW[1] == CCW[7].
        let cw_1 = FluxSpinner::new(1).spin(Spin::Clockwise).build_lines();
        let ccw_7 = FluxSpinner::new(7)
            .spin(Spin::CounterClockwise)
            .build_lines();
        assert_eq!(cw_1, ccw_7, "CW tick 1 should equal CCW tick 7");
    }

    #[test]
    fn ccw_wraps_after_eight_steps() {
        let f0 = FluxSpinner::new(0)
            .spin(Spin::CounterClockwise)
            .build_lines();
        let f8 = FluxSpinner::new(8)
            .spin(Spin::CounterClockwise)
            .build_lines();
        assert_eq!(f0, f8, "CCW should wrap back to frame 0 after 8 ticks");
    }

    #[test]
    fn ccw_wave_differs_from_cw_wave() {
        let cw = FluxSpinner::new(1)
            .width(4)
            .phase_step(1)
            .spin(Spin::Clockwise)
            .build_lines();
        let ccw = FluxSpinner::new(1)
            .width(4)
            .phase_step(1)
            .spin(Spin::CounterClockwise)
            .build_lines();
        assert_ne!(
            cw, ccw,
            "CW and CCW waves should differ for width>1 at tick 1"
        );
    }

    // ── Phase wave ────────────────────────────────────────────────────────────

    #[test]
    fn phase_step_zero_all_cells_same() {
        let lines = FluxSpinner::new(0).width(4).phase_step(0).build_lines();
        let spans = &lines[0].spans;
        let first = &spans[0].content;
        for s in spans.iter().skip(1) {
            assert_eq!(&s.content, first, "phase_step=0 → all cells identical");
        }
    }

    #[test]
    fn phase_step_one_cells_differ() {
        let lines = FluxSpinner::new(0).width(4).phase_step(1).build_lines();
        let spans = &lines[0].spans;
        for pair in spans.windows(2) {
            assert_ne!(
                pair[0].content, pair[1].content,
                "adjacent cells should differ with phase_step=1"
            );
        }
    }

    #[test]
    fn phase_step_eight_wraps_to_same() {
        let base = FluxSpinner::new(0).width(3).phase_step(0).build_lines();
        let wrap = FluxSpinner::new(0).width(3).phase_step(8).build_lines();
        assert_eq!(base, wrap, "phase_step=8 should behave like phase_step=0");
    }

    // ── Size ──────────────────────────────────────────────────────────────────

    #[test]
    fn output_dimensions_match_width_height() {
        for w in 1..=5usize {
            for h in 1..=3usize {
                let lines = FluxSpinner::new(0).width(w).height(h).build_lines();
                assert_eq!(lines.len(), h, "height={h}");
                for (i, line) in lines.iter().enumerate() {
                    assert_eq!(line.spans.len(), w, "row {i}: width={w}");
                }
            }
        }
    }

    #[test]
    fn char_size_returns_width_height() {
        let s = FluxSpinner::new(0).width(4).height(2);
        assert_eq!(s.char_size(), (4, 2));
    }

    #[test]
    fn char_size_clamps_to_one() {
        let s = FluxSpinner::new(0).width(0).height(0);
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
                frame.render_widget(FluxSpinner::new(42).width(3).height(2), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn widget_single_char_renders() {
        let backend = TestBackend::new(1, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(FluxSpinner::new(0), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn widget_zero_area_no_panic() {
        let backend = TestBackend::new(0, 0);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(FluxSpinner::new(0), frame.area());
            })
            .unwrap();
    }

    // ── Builder chain ─────────────────────────────────────────────────────────

    #[test]
    fn builder_chain() {
        use ratatui::widgets::Block;
        let s = FluxSpinner::new(99)
            .width(6)
            .height(2)
            .spin(Spin::CounterClockwise)
            .color(Color::Green)
            .ticks_per_step(3)
            .phase_step(2)
            .frames(FluxFrames::LINE)
            .block(Block::bordered())
            .alignment(Alignment::Center);
        assert_eq!(s.width, 6);
        assert_eq!(s.height, 2);
        assert!(matches!(s.spin, Spin::CounterClockwise));
        assert_eq!(s.color, Color::Green);
        assert_eq!(s.ticks_per_step, 3);
        assert_eq!(s.phase_step, 2);
        assert_eq!(s.frames, FluxFrames::LINE);
    }

    #[test]
    fn output_chars_come_from_frame_set() {
        for tick in 0..8u64 {
            for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                let lines = FluxSpinner::new(tick)
                    .width(4)
                    .height(2)
                    .spin(spin)
                    .build_lines();
                let frame_set: std::collections::HashSet<char> =
                    FluxFrames::BRAILLE.iter().copied().collect();
                for line in &lines {
                    for span in &line.spans {
                        let ch = span.content.chars().next().unwrap();
                        assert!(
                            frame_set.contains(&ch),
                            "U+{:04X} not in BRAILLE preset",
                            ch as u32
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn custom_frames_respected() {
        let frames: &'static [char] = &['a', 'b', 'c', 'd'];
        let lines = FluxSpinner::new(0).frames(frames).build_lines();
        let ch = lines[0].spans[0].content.chars().next().unwrap();
        assert_eq!(ch, 'a', "tick=0 should show first custom frame");

        let lines4 = FluxSpinner::new(4).frames(frames).build_lines();
        let ch4 = lines4[0].spans[0].content.chars().next().unwrap();
        assert_eq!(ch4, 'a', "tick=4 (n=4) wraps back to first frame");
    }

    #[test]
    fn frames_builder_changes_output() {
        let braille = FluxSpinner::new(1)
            .frames(FluxFrames::BRAILLE)
            .build_lines();
        let line = FluxSpinner::new(1).frames(FluxFrames::LINE).build_lines();
        assert_ne!(
            braille, line,
            "different frame sets produce different output"
        );
    }

    #[test]
    fn to_lines_matches_build_lines() {
        let s = FluxSpinner::new(3).width(6).height(2);
        assert_eq!(s.to_lines(), s.build_lines());
    }

    #[test]
    fn to_text_has_expected_row_count() {
        let s = FluxSpinner::new(0).width(4).height(3);
        assert_eq!(s.to_text().lines.len(), 3);
    }

    #[test]
    fn converts_into_text_and_cell() {
        use ratatui::text::Text;
        use ratatui::widgets::Cell;

        let s = FluxSpinner::new(2).width(5).height(2);
        // From<&Spinner> and From<Spinner> for Text
        let by_ref: Text = (&s).into();
        let by_val: Text = s.clone().into();
        assert_eq!(by_ref.lines.len(), 2);
        assert_eq!(by_ref, by_val);
        // Cell accepts anything Into<Text>
        let _cell_ref = Cell::from(&s);
        let _cell_val = Cell::from(s);
    }
}
