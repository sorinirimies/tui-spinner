//! Linear spinner — horizontal scrolling window or vertical bounce.
//!
//! A single [`LinearSpinner`] covers both animation patterns along a straight axis:
//!
//! - **[`Direction::Horizontal`]** — a window of lit symbols scrolls left-to-right
//!   across a row of configurable length, wrapping around. Classic ellipsis effect.
//!
//! - **[`Direction::Vertical`]** — a single lit symbol bounces up and down a column
//!   of configurable height: `0 → 1 → … → n-1 → … → 1 → 0 → …`
//!   (the "Zed / Copilot" activity indicator pattern).
//!
//! Both directions support the same set of [`LinearStyle`] symbol pairs, so you
//! can mix and match appearance independently of layout direction.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

// ── Direction ─────────────────────────────────────────────────────────────────

/// The animation direction (and layout axis) of a [`LinearSpinner`].
///
/// # Examples
///
/// ```
/// use tui_spinner::{LinearSpinner, Direction};
///
/// let horizontal = LinearSpinner::new(0).direction(Direction::Horizontal);
/// let vertical   = LinearSpinner::new(0).direction(Direction::Vertical);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// A window of lit symbols scrolls left-to-right across a single row.
    ///
    /// The widget occupies **1 row** of height and `total_slots` columns of width.
    #[default]
    Horizontal,

    /// A single lit symbol bounces up and down a single column.
    ///
    /// The widget occupies **1 column** of width and `total_slots` rows of height.
    /// When the area is taller than `total_slots` the symbols are pinned to the
    /// bottom so they align with the last log line in a side-column layout.
    Vertical,
}

// ── Flow ──────────────────────────────────────────────────────────────────────

/// The animation flow direction of a [`LinearSpinner`].
///
/// Controls whether the animation plays forwards (the default) or backwards.
///
/// - [`Flow::Forwards`] — horizontal scrolls left-to-right; vertical bounces
///   starting upward (index 0 → n-1 → 0 …).
/// - [`Flow::Backwards`] — horizontal scrolls right-to-left; vertical bounces
///   starting downward (index n-1 → 0 → n-1 …).
///
/// # Examples
///
/// ```
/// use tui_spinner::{LinearSpinner, Flow};
///
/// let backwards = LinearSpinner::new(0).flow(Flow::Backwards);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Flow {
    /// Normal playback direction (default).
    ///
    /// Horizontal scrolls left-to-right; vertical bounces starting from the top.
    #[default]
    Forwards,

    /// Reversed playback direction.
    ///
    /// Horizontal scrolls right-to-left; vertical bounces starting from the bottom.
    Backwards,
}

// ── LinearStyle ──────────────────────────────────────────────────────────────────

/// The symbol pair used to draw active and inactive slot positions.
///
/// Each variant defines an `(active, inactive)` character pair rendered with
/// bold + `active_color` / `inactive_color` respectively.
///
/// # Examples
///
/// ```
/// use tui_spinner::{LinearSpinner, LinearStyle};
///
/// let spinner = LinearSpinner::new(0).linear_style(LinearStyle::Diamond);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinearStyle {
    /// `●` / `·` — filled circle / middle dot. The original classic look.
    #[default]
    Classic,

    /// `■` / `□` — filled square / open square.
    Square,

    /// `◆` / `◇` — filled diamond / open diamond.
    Diamond,

    /// `▰` / `▱` — parallelogram filled / parallelogram empty.
    Bar,

    /// `⣿` / `⠀` — full braille cell / blank braille cell.
    Braille,

    /// `▶` / `▷` — filled right-arrow / open right-arrow.
    /// Rotates to `▼` / `▽` when [`Direction::Vertical`] is used.
    Arrow,
}

impl LinearStyle {
    /// Returns the `(active, inactive)` string pair for this style,
    /// optionally adjusted for the given direction.
    #[must_use]
    pub const fn symbols(self, direction: Direction) -> (&'static str, &'static str) {
        match self {
            Self::Classic => ("●", "·"),
            Self::Square => ("■", "□"),
            Self::Diamond => ("◆", "◇"),
            Self::Bar => ("▰", "▱"),
            Self::Braille => ("⣿", "⠀"),
            Self::Arrow => match direction {
                Direction::Horizontal => ("▶", "▷"),
                Direction::Vertical => ("▼", "▽"),
            },
        }
    }

    /// Returns the number of terminal columns each slot occupies.
    ///
    /// Most symbols are 1 column wide in a typical Western terminal.
    /// Symbols whose Unicode East Asian Width property is "Wide" occupy 2
    /// columns; callers that lay out the spinner area manually (e.g. in
    /// a Ratatui [`Layout`]) should multiply `total_slots` by this value
    /// to get the correct `Constraint::Length` value.
    ///
    /// [`Layout`]: ratatui::layout::Layout
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{Direction, LinearStyle};
    ///
    /// // Allocate exactly the right width for a 5-slot horizontal spinner:
    /// let style = LinearStyle::Classic;
    /// let width = 5 * style.columns_per_slot();
    /// assert_eq!(width, 5);
    /// ```
    #[must_use]
    pub const fn columns_per_slot(self) -> u16 {
        // All current styles occupy exactly 1 terminal column.
        // EAW=N (Narrow) and EAW=A (Ambiguous) symbols are both treated as
        // 1 column in Western terminals; update individual arms here if a
        // future style adds a genuinely Wide (EAW=W) glyph.
        match self {
            Self::Classic
            | Self::Square
            | Self::Diamond
            | Self::Bar
            | Self::Braille
            | Self::Arrow => 1,
        }
    }
}

// ── LinearSpinner ───────────────────────────────────────────────────────────────

/// A linear spinner that animates either horizontally or vertically.
///
/// Pass a monotonically increasing `tick` counter (typically incremented once
/// per render frame) and call `.direction()`, `.linear_style()`, and colour
/// methods to customise the appearance.
///
/// # Horizontal (default)
///
/// ```text
/// tick  0–2 : ●●·
/// tick  3–5 : ·●●
/// tick  6–8 : ··●   (window wraps)
/// tick  9–11: ●··
/// ```
///
/// # Vertical (bounce)
///
/// ```text
/// tick  0–2 : ●      ← slot 0 lit
///             ·
///             ·
/// tick  3–5 : ·
///             ●      ← slot 1 lit
///             ·
/// tick  6–8 : ·
///             ·
///             ●      ← slot 2 lit
/// tick  9–11: ·
///             ●      ← slot 1 lit (bouncing back)
///             ·
/// ```
///
/// # Examples
///
/// ```no_run
/// use ratatui::Frame;
/// use ratatui::layout::Rect;
/// use tui_spinner::{Direction, LinearStyle, LinearSpinner};
///
/// fn draw(frame: &mut Frame, area: Rect, tick: u64) {
///     // Horizontal ellipsis
///     frame.render_widget(LinearSpinner::new(tick), area);
///
///     // Vertical bounce with diamond symbols
///     frame.render_widget(
///         LinearSpinner::new(tick)
///             .direction(Direction::Vertical)
///             .linear_style(LinearStyle::Diamond),
///         area,
///     );
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct LinearSpinner<'a> {
    /// Monotonically increasing frame counter.
    tick: u64,
    /// Total number of slots.
    total_slots: usize,
    /// How many consecutive slots are lit at once (horizontal only).
    lit_slots: usize,
    /// Ticks each animation step is held (default: 3).
    ticks_per_step: u64,
    /// Animation direction and layout axis.
    direction: Direction,
    /// Animation flow (forwards or backwards).
    flow: Flow,
    /// Symbol set.
    linear_style: LinearStyle,
    /// Colour of lit / active symbols.
    active_color: Color,
    /// Colour of unlit / inactive symbols.
    inactive_color: Color,
    /// Optional block wrapper.
    block: Option<Block<'a>>,
    /// Base style.
    style: Style,
}

impl<'a> LinearSpinner<'a> {
    /// Creates a new [`LinearSpinner`] at the given animation tick with all
    /// defaults: 3 slots, 2 lit, horizontal, classic style, 3 ticks/step.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::LinearSpinner;
    ///
    /// let spinner = LinearSpinner::new(0);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            total_slots: 3,
            lit_slots: 2,
            ticks_per_step: 3,
            direction: Direction::Horizontal,
            flow: Flow::Forwards,
            linear_style: LinearStyle::Classic,
            active_color: Color::White,
            inactive_color: Color::DarkGray,
            block: None,
            style: Style::default(),
        }
    }

    /// Sets the animation direction (default: [`Direction::Horizontal`]).
    ///
    /// - [`Direction::Horizontal`] — scrolling window across a row.
    /// - [`Direction::Vertical`]   — bouncing symbol down a column.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{Direction, LinearSpinner};
    ///
    /// let vertical = LinearSpinner::new(0).direction(Direction::Vertical);
    /// ```
    #[must_use]
    pub const fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the animation flow direction (default: [`Flow::Forwards`]).
    ///
    /// - [`Flow::Forwards`]  — normal playback (left-to-right / upward-first bounce).
    /// - [`Flow::Backwards`] — reversed playback (right-to-left / downward-first bounce).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{Flow, LinearSpinner};
    ///
    /// let backwards = LinearSpinner::new(0).flow(Flow::Backwards);
    /// ```
    #[must_use]
    pub const fn flow(mut self, flow: Flow) -> Self {
        self.flow = flow;
        self
    }

    /// Sets the symbol pair used to draw active and inactive slots
    /// (default: [`LinearStyle::Classic`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{LinearStyle, LinearSpinner};
    ///
    /// let spinner = LinearSpinner::new(0).linear_style(LinearStyle::Square);
    /// ```
    #[must_use]
    pub const fn linear_style(mut self, style: LinearStyle) -> Self {
        self.linear_style = style;
        self
    }

    /// Sets the total number of slots (default: 3, minimum: 1).
    ///
    /// For [`Direction::Vertical`] this is the column height.
    /// For [`Direction::Horizontal`] this is the row width.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::LinearSpinner;
    ///
    /// let spinner = LinearSpinner::new(0).total_slots(5);
    /// ```
    #[must_use]
    pub fn total_slots(mut self, n: usize) -> Self {
        self.total_slots = n.max(1);
        self
    }

    /// Sets the number of consecutive slots lit at once (default: 2).
    ///
    /// Only meaningful for [`Direction::Horizontal`]; ignored in vertical mode
    /// where exactly one slot is always lit. Values are clamped at render time
    /// to `[1, total_slots]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::LinearSpinner;
    ///
    /// let spinner = LinearSpinner::new(0).lit_slots(1);
    /// ```
    #[must_use]
    pub fn lit_slots(mut self, n: usize) -> Self {
        self.lit_slots = n.max(1);
        self
    }

    /// Sets how many ticks each animation step is held (default: 3).
    ///
    /// Higher values slow the animation; lower values speed it up. Zero is
    /// silently clamped to 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::LinearSpinner;
    ///
    /// let fast = LinearSpinner::new(0).ticks_per_step(1);
    /// ```
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.ticks_per_step = n.max(1);
        self
    }

    /// Sets the colour of active (lit) symbols (default: [`Color::White`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::LinearSpinner;
    ///
    /// let spinner = LinearSpinner::new(0).active_color(Color::Cyan);
    /// ```
    #[must_use]
    pub const fn active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    /// Sets the colour of inactive (dim) symbols (default: [`Color::DarkGray`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::LinearSpinner;
    ///
    /// let spinner = LinearSpinner::new(0).inactive_color(Color::DarkGray);
    /// ```
    #[must_use]
    pub const fn inactive_color(mut self, color: Color) -> Self {
        self.inactive_color = color;
        self
    }

    /// Wraps the spinner in a [`Block`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::widgets::Block;
    /// use tui_spinner::LinearSpinner;
    ///
    /// let spinner = LinearSpinner::new(0).block(Block::bordered().title("Loading"));
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

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Current animation step index (ticks divided and wrapped to `total_slots`).
    #[allow(clippy::cast_possible_truncation)]
    fn step(&self) -> usize {
        (self.tick / self.ticks_per_step) as usize
    }

    /// Span for one slot at `idx` — shared by both directions.
    fn slot_span(&self, _idx: usize, is_lit: bool) -> Span<'static> {
        let (on, off) = self.linear_style.symbols(self.direction);
        if is_lit {
            Span::styled(
                on,
                Style::default()
                    .fg(self.active_color)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(off, Style::default().fg(self.inactive_color))
        }
    }

    // ── Horizontal rendering ──────────────────────────────────────────────────

    /// Builds the single [`Line`] used in horizontal mode.
    fn build_horizontal_line(&self) -> Line<'static> {
        let total = self.total_slots.max(1);
        let lit = self.lit_slots.min(total);
        let raw_step = self.step() % total;

        // In backwards mode the step index runs in reverse so the window
        // scrolls right-to-left instead of left-to-right.
        let step = match self.flow {
            Flow::Forwards => raw_step,
            Flow::Backwards => (total - 1) - raw_step,
        };

        let spans: Vec<Span<'static>> = (0..total)
            .map(|i| {
                let is_lit = if step + lit <= total {
                    i >= step && i < step + lit
                } else {
                    // Window wraps around the end.
                    i >= step || i < (step + lit) % total
                };
                self.slot_span(i, is_lit)
            })
            .collect();

        Line::from(spans)
    }

    // ── Vertical rendering ────────────────────────────────────────────────────

    /// The bounce sequence maps a step index to a slot index.
    ///
    /// For `total_slots = n` the forwards sequence is `0, 1, …, n-1, n-2, …, 1`
    /// (ping-pong starting from the top).  [`Flow::Backwards`] mirrors this to
    /// `n-1, n-2, …, 0, 1, …, n-2` (starting from the bottom).
    fn bounce_index(&self) -> usize {
        let n = self.total_slots.max(1);
        if n == 1 {
            return 0;
        }
        // Full cycle length = 2*(n-1)
        let cycle = 2 * (n - 1);
        let pos = self.step() % cycle;
        let idx = if pos < n { pos } else { cycle - pos };

        match self.flow {
            Flow::Forwards => idx,
            // Mirror: 0 ↔ n-1, 1 ↔ n-2, …
            Flow::Backwards => (n - 1) - idx,
        }
    }

    /// Builds the column of [`Line`]s used in vertical mode, bottom-aligned
    /// within the available `height`.
    fn build_vertical_lines(&self, height: usize) -> Vec<Line<'static>> {
        let n = self.total_slots.max(1);
        let active = self.bounce_index();

        let mut lines: Vec<Line<'static>> = vec![Line::from(""); height];

        // Pin the symbols to the last `n` rows so they sit next to the latest
        // log line when used as a side-column activity indicator.
        if height >= n {
            let start = height - n;
            for (i, line) in lines.iter_mut().skip(start).enumerate() {
                *line = Line::from(self.slot_span(i, i == active));
            }
        } else {
            // Area shorter than slot count — fill all rows in order.
            for (i, line) in lines.iter_mut().enumerate() {
                *line = Line::from(self.slot_span(i, i == active));
            }
        }

        lines
    }
}

// ── Styled ────────────────────────────────────────────────────────────────────

impl_styled_for!(LinearSpinner<'_>);

// ── Widget ────────────────────────────────────────────────────────────────────

impl_widget_via_ref!(LinearSpinner<'_>);

impl Widget for &LinearSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        let inner = if let Some(ref block) = self.block {
            let inner_area = block.inner(area);
            block.clone().render(area, buf);
            inner_area
        } else {
            area
        };

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        match self.direction {
            Direction::Horizontal => {
                Paragraph::new(self.build_horizontal_line()).render(inner, buf);
            }
            Direction::Vertical => {
                let lines = self.build_vertical_lines(inner.height as usize);
                Paragraph::new(lines).render(inner, buf);
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    use super::*;

    // ── Horizontal ────────────────────────────────────────────────────────────

    #[test]
    fn horizontal_first_step_lights_first_two() {
        let s = LinearSpinner::new(0); // step=0, lit=2 → slots 0,1
        let line = s.build_horizontal_line();
        let content: Vec<&str> = line.spans.iter().map(|sp| sp.content.as_ref()).collect();
        let (on, off) = LinearStyle::Classic.symbols(Direction::Horizontal);
        assert_eq!(content, &[on, on, off]);
    }

    #[test]
    fn horizontal_second_step_advances() {
        let s = LinearSpinner::new(3); // step=1 → slots 1,2
        let line = s.build_horizontal_line();
        let content: Vec<&str> = line.spans.iter().map(|sp| sp.content.as_ref()).collect();
        let (on, off) = LinearStyle::Classic.symbols(Direction::Horizontal);
        assert_eq!(content, &[off, on, on]);
    }

    #[test]
    fn horizontal_window_wraps() {
        let s = LinearSpinner::new(6); // step=2, lit=2 → slots 2 and 0 (wrap)
        let line = s.build_horizontal_line();
        let content: Vec<&str> = line.spans.iter().map(|sp| sp.content.as_ref()).collect();
        let (on, off) = LinearStyle::Classic.symbols(Direction::Horizontal);
        assert_eq!(content, &[on, off, on]);
    }

    #[test]
    fn horizontal_lit_clamped_to_total_at_render() {
        let s = LinearSpinner::new(0).total_slots(2).lit_slots(99);
        let line = s.build_horizontal_line();
        let (on, _) = LinearStyle::Classic.symbols(Direction::Horizontal);
        let lit = line
            .spans
            .iter()
            .filter(|sp| sp.content.as_ref() == on)
            .count();
        assert!(lit <= 2, "lit count must not exceed total_slots");
    }

    #[test]
    fn horizontal_single_dot_no_panic() {
        let s = LinearSpinner::new(0).total_slots(1).lit_slots(1);
        let _ = s.build_horizontal_line();
    }

    // ── Vertical / bounce ─────────────────────────────────────────────────────

    #[test]
    fn vertical_bounce_sequence_3_dots() {
        // For n=3 the cycle is [0,1,2,1] with ticks_per_step=3.
        let expected = [0usize, 0, 0, 1, 1, 1, 2, 2, 2, 1, 1, 1, 0, 0, 0];
        for (tick, &exp) in expected.iter().enumerate() {
            let s = LinearSpinner::new(tick as u64).direction(Direction::Vertical);
            assert_eq!(s.bounce_index(), exp, "tick={tick}");
        }
    }

    #[test]
    fn vertical_bounce_sequence_1_dot() {
        // Single slot — always index 0, never panics.
        for tick in 0..10u64 {
            let s = LinearSpinner::new(tick)
                .direction(Direction::Vertical)
                .total_slots(1);
            assert_eq!(s.bounce_index(), 0);
        }
    }

    // ── Flow::Backwards — horizontal ──────────────────────────────────────────

    #[test]
    fn horizontal_backwards_first_step_lights_last_two() {
        // Flow::Backwards with step=0 → reversed step = total-1 = 2, lit=2
        // so the window starts at index 2 and wraps: slots 2 and 0 are lit.
        let s = LinearSpinner::new(0).flow(Flow::Backwards);
        let line = s.build_horizontal_line();
        let content: Vec<&str> = line.spans.iter().map(|sp| sp.content.as_ref()).collect();
        let (on, off) = LinearStyle::Classic.symbols(Direction::Horizontal);
        assert_eq!(content, &[on, off, on]);
    }

    #[test]
    fn horizontal_backwards_second_step_reverses() {
        // Flow::Backwards, tick=3 → raw step=1 → reversed step = 2-1 = 1, lit=2
        // window at 1: slots 1,2 lit.
        let s = LinearSpinner::new(3).flow(Flow::Backwards);
        let line = s.build_horizontal_line();
        let content: Vec<&str> = line.spans.iter().map(|sp| sp.content.as_ref()).collect();
        let (on, off) = LinearStyle::Classic.symbols(Direction::Horizontal);
        assert_eq!(content, &[off, on, on]);
    }

    #[test]
    fn horizontal_backwards_third_step() {
        // Flow::Backwards, tick=6 → raw step=2 → reversed step = 2-2 = 0, lit=2
        // window at 0: slots 0,1 lit.
        let s = LinearSpinner::new(6).flow(Flow::Backwards);
        let line = s.build_horizontal_line();
        let content: Vec<&str> = line.spans.iter().map(|sp| sp.content.as_ref()).collect();
        let (on, off) = LinearStyle::Classic.symbols(Direction::Horizontal);
        assert_eq!(content, &[on, on, off]);
    }

    // ── Flow::Backwards — vertical / bounce ───────────────────────────────────

    #[test]
    fn vertical_backwards_bounce_sequence_3_dots() {
        // For n=3 forwards bounce is [0,1,2,1]. Backwards mirrors: [2,1,0,1].
        // With ticks_per_step=3, each step is held for 3 ticks.
        let expected = [2usize, 2, 2, 1, 1, 1, 0, 0, 0, 1, 1, 1, 2, 2, 2];
        for (tick, &exp) in expected.iter().enumerate() {
            let s = LinearSpinner::new(tick as u64)
                .direction(Direction::Vertical)
                .flow(Flow::Backwards);
            assert_eq!(s.bounce_index(), exp, "tick={tick}");
        }
    }

    #[test]
    fn vertical_backwards_bounce_sequence_1_dot() {
        // Single slot — always index 0 regardless of flow.
        for tick in 0..10u64 {
            let s = LinearSpinner::new(tick)
                .direction(Direction::Vertical)
                .flow(Flow::Backwards)
                .total_slots(1);
            assert_eq!(s.bounce_index(), 0);
        }
    }

    #[test]
    fn flow_forwards_is_default() {
        let s = LinearSpinner::new(0);
        assert_eq!(s.flow, Flow::Forwards);
    }

    #[test]
    fn flow_default_trait() {
        assert_eq!(Flow::default(), Flow::Forwards);
    }

    #[test]
    fn vertical_ticks_per_step_one_faster() {
        let s = LinearSpinner::new(1)
            .direction(Direction::Vertical)
            .ticks_per_step(1);
        assert_eq!(s.bounce_index(), 1);
    }

    #[test]
    fn vertical_lines_bottom_aligned() {
        let s = LinearSpinner::new(0)
            .direction(Direction::Vertical)
            .total_slots(3);
        let lines = s.build_vertical_lines(6);
        // First 3 rows should be empty, last 3 should contain symbols.
        assert_eq!(lines.len(), 6);
        assert!(lines[0].spans.is_empty() || lines[0].to_string().is_empty());
        assert!(!lines[3].spans.is_empty());
    }

    #[test]
    fn vertical_lines_short_area_no_panic() {
        let s = LinearSpinner::new(0)
            .direction(Direction::Vertical)
            .total_slots(5);
        let lines = s.build_vertical_lines(2);
        assert_eq!(lines.len(), 2);
    }

    // ── LinearStyle symbols ──────────────────────────────────────────────────────

    #[test]
    fn linear_style_arrow_changes_with_direction() {
        let (h_on, _) = LinearStyle::Arrow.symbols(Direction::Horizontal);
        let (v_on, _) = LinearStyle::Arrow.symbols(Direction::Vertical);
        assert_ne!(h_on, v_on, "Arrow should differ between H and V");
    }

    #[test]
    fn all_styles_return_non_empty_symbols() {
        let styles = [
            LinearStyle::Classic,
            LinearStyle::Square,
            LinearStyle::Diamond,
            LinearStyle::Bar,
            LinearStyle::Braille,
            LinearStyle::Arrow,
        ];
        for style in styles {
            for dir in [Direction::Horizontal, Direction::Vertical] {
                let (on, off) = style.symbols(dir);
                assert!(!on.is_empty(), "{style:?}/{dir:?} active symbol empty");
                assert!(!off.is_empty(), "{style:?}/{dir:?} inactive symbol empty");
            }
        }
    }

    // ── Widget render smoke tests ─────────────────────────────────────────────

    #[test]
    fn render_horizontal_does_not_panic_on_zero_area() {
        let s = LinearSpinner::new(0);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        Widget::render(&s, area, &mut buf);
    }

    #[test]
    fn render_vertical_does_not_panic_on_zero_area() {
        let s = LinearSpinner::new(0).direction(Direction::Vertical);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        Widget::render(&s, area, &mut buf);
    }

    #[test]
    fn render_vertical_does_not_panic_on_small_area() {
        let s = LinearSpinner::new(5).direction(Direction::Vertical);
        let area = Rect::new(0, 0, 1, 1);
        let mut buf = Buffer::empty(area);
        Widget::render(&s, area, &mut buf);
    }
}
