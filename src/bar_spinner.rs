//! Rectangle braille-arc bouncing spinner.
//!
//! A braille or symbol-based loading bar: every character cell is filled with a
//! glyph; a bright arc window slides left-to-right (and bounces) over
//! a dimmer background track.
//!
//! Set [`BarSpinner::width`] to `0` (the default) to fill the
//! available terminal width automatically.
//!
//! ## Visual
//!
//! ```text
//! ⣀⣀⣀⣀⠉⠛⠿⣿⣿⣿⣿⣿⣿⠿⠛⠉⣀⣀⣀⣀⣀⣀⣀⣀
//! ```
//!
//! The outer-edge columns of the arc (`⠉ ⠛ ⠿`) taper from a short
//! top-row glyph up to full density, giving a smooth gradient.
//! The dim background track uses `⣀` (bottom-two-dot rail).
//!
//! ## How it works
//!
//! 1. A `width × height` character grid is rendered; arc columns use the
//!    full-dot glyph `⣿` in `arc_color`, dim columns use `⣀` in `dim_color`.
//! 2. The three outermost columns on each arc edge use the fade ramp
//!    `⠉ ⠛ ⠿` so the arc blends into the track.
//! 3. The arc window advances one column per step and reverses at each end.

use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style, Styled};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::Spin;

// ── Braille glyph constants ───────────────────────────────────────────────────

/// Fade ramp for arc edges — outermost (index 0) to innermost (index 3).
///
/// ```text
/// ⠉  0x09   dots 1,4       — top row only
/// ⠛  0x1B   dots 1,2,4,5   — top two rows
/// ⠿  0x3F   dots 1–6       — top three rows
/// ⣿  0xFF   dots 1–8       — full
/// ```
const FADE: [u8; 4] = [0x09, 0x1B, 0x3F, 0xFF];

/// Braille byte for the dim background track.
///
/// `⣀` (0xC0) — bottom two dots only — gives a subtle rail behind the arc.
const DIM_BYTE: u8 = 0xC0;

// ── Track style ───────────────────────────────────────────────────────────────

/// Controls the appearance of the dim background track behind the bouncing arc.
///
/// | Variant | Byte | Glyph | Effect |
/// |---------|------|-------|--------|
/// | `Rail`  | `0xC0` | `⣀` | Bottom-two-dot baseline — subtle, default |
/// | `Full`  | `0xFF` | `⣿` | Full-density track in `dim_color` |
/// | `Empty` | `0x00` | `⠀` | Invisible — arc floats on empty space |
/// | `Custom(u8)` | any | any braille | User-defined braille byte |
///
/// # Examples
///
/// ```
/// use tui_spinner::{BarSpinner, BarTrack};
///
/// let rail  = BarSpinner::new(0).track(BarTrack::Rail);    // ⣀ default
/// let solid = BarSpinner::new(0).track(BarTrack::Full);    // ⣿ solid track
/// let float = BarSpinner::new(0).track(BarTrack::Empty);   // ⠀ no track
/// let dot   = BarSpinner::new(0).track(BarTrack::Custom(0x09)); // ⠉ top-row
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarTrack {
    /// `⣀` (0xC0) — bottom-two-dot rail, subtle baseline (default).
    #[default]
    Rail,
    /// `⣿` (0xFF) — full-density track in `dim_color`.
    Full,
    /// `⠀` (0x00) — invisible background; the arc floats on empty space.
    Empty,
    /// Any custom braille byte.
    Custom(u8),
}

impl BarTrack {
    fn byte(self) -> u8 {
        match self {
            Self::Rail => DIM_BYTE,
            Self::Full => 0xFF,
            Self::Empty => 0x00,
            Self::Custom(b) => b,
        }
    }
}

// ── Symbol style ─────────────────────────────────────────────────────────────

/// Selects the glyph set used for the arc and background track.
///
/// [`Braille`](BarStyle::Braille) (the default) uses braille bytes and
/// supports the full `fade_width` density ramp.  All other variants use a
/// single Unicode character for the arc and one for the track, with no
/// intermediate fade.
///
/// | Variant   | Arc | Track | Notes |
/// |-----------|-----|-------|-------|
/// | `Braille` | `⣿` | `⣀`  | Braille density fade (default) |
/// | `Block`   | `█` | `░`   | Solid / light block |
/// | `Shade`   | `▓` | `░`   | Dark shade / light block |
/// | `Dot`     | `●` | `·`   | Filled / middle dot |
/// | `Diamond` | `◆` | `◇`   | Filled / open diamond |
/// | `Square`  | `■` | `□`   | Filled / open square |
/// | `Star`    | `★` | `☆` | Filled / outline star             |
/// | `Heart`   | `♥` | `♡` | Filled / outline heart            |
/// | `Arrow`   | `▶` | `▷` | Solid / outline right triangle    |
/// | `Circle`  | `◉` | `○` | Fisheye / open circle             |
/// | `Spark`   | `✦` | `✧` | Black / white four-pointed star   |
/// | `Cross`    | `✚` | `✛` | Heavy / open-centre cross         |
/// | `Progress` | `▰` | `▱` | Bold progress-bar segments |
/// | `Thick`    | `━` | `─` | Heavy / thin horizontal line |
/// | `Wave`     | `≈` | `˜` | Wave / tilde |
/// | `Pip`      | `▪` | `·` | Small square / middle dot |
///
/// # Examples
///
/// ```
/// use tui_spinner::{BarSpinner, BarStyle};
///
/// let braille = BarSpinner::new(0);                                    // default
/// let block   = BarSpinner::new(0).bar_style(BarStyle::Block);
/// let dot     = BarSpinner::new(0).bar_style(BarStyle::Dot);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarStyle {
    /// Dense braille glyphs with a smooth density-gradient fade (default).
    ///
    /// Respects `arc_char`, `track`, and `fade_width` settings.
    #[default]
    Braille,
    /// Solid block — `█` arc, `░` track.
    Block,
    /// Shade blocks — `▓` arc, `░` track.
    Shade,
    /// Filled / middle dot — `●` arc, `·` track.
    Dot,
    /// Filled / open diamond — `◆` arc, `◇` track.
    Diamond,
    /// Filled / open square — `■` arc, `□` track.
    Square,
    /// Filled / outline star — `★` arc, `☆` track.
    Star,
    /// Filled / outline heart — `♥` arc, `♡` track.
    Heart,
    /// Solid / outline right-pointing triangle — `▶` arc, `▷` track.
    Arrow,
    /// Fisheye / open circle — `◉` arc, `○` track.
    Circle,
    /// Black / white four-pointed star — `✦` arc, `✧` track.
    Spark,
    /// Heavy Greek cross / open-centre cross — `✚` arc, `✛` track.
    Cross,
    /// Bold progress-bar segments — `▰` arc, `▱` track.
    Progress,
    /// Heavy / thin horizontal line — `━` arc, `─` track.
    Thick,
    /// Wave / tilde — `≈` arc, `˜` track.
    Wave,
    /// Small square / middle dot — `▪` arc, `·` track.
    Pip,
}

impl BarStyle {
    /// Returns `Some((arc_char, track_char))` for symbol styles, or `None`
    /// for [`BarStyle::Braille`] (which uses the existing braille rendering).
    pub(crate) fn chars(self) -> Option<(char, char)> {
        match self {
            Self::Braille => None,
            Self::Block => Some(('█', '░')),
            Self::Shade => Some(('▓', '░')),
            Self::Dot => Some(('●', '·')),
            Self::Diamond => Some(('◆', '◇')),
            Self::Square => Some(('■', '□')),
            Self::Star => Some(('★', '☆')),
            Self::Heart => Some(('♥', '♡')),
            Self::Arrow => Some(('▶', '▷')),
            Self::Circle => Some(('◉', '○')),
            Self::Spark => Some(('✦', '✧')),
            Self::Cross => Some(('✚', '✛')),
            Self::Progress => Some(('▰', '▱')),
            Self::Thick => Some(('━', '─')),
            Self::Wave => Some(('≈', '˜')),
            Self::Pip => Some(('▪', '·')),
        }
    }
}

// ── Engine ────────────────────────────────────────────────────────────────────

/// Map an edge distance to the appropriate [`FADE`] byte.
///
/// - `fade_width = 0` → no ramp; all arc cells use `arc_byte`.
/// - `fade_width = 1` → only the outermost column fades.
/// - `fade_width = 3` → default three-column ramp (`⠉ ⠛ ⠿ → arc_byte`).
#[inline]
fn fade_byte(from_edge: usize, fade_width: usize, arc_byte: u8) -> u8 {
    if fade_width == 0 || from_edge >= fade_width {
        arc_byte
    } else {
        FADE[(from_edge * 3).div_ceil(fade_width).min(2)]
    }
}

/// Internal bounce engine operating in **character-column** space.
///
/// All positions are whole character columns, so there are never any
/// sub-character boundary artefacts.
struct RectEngine {
    char_w: usize,
    char_h: usize,
    /// Width of the bright window in character columns.
    arc_cols: usize,
    /// Leftmost character column of the bright window.
    anchor: usize,
    going_forward: bool,
}

impl RectEngine {
    fn build(char_w: usize, char_h: usize, arc_width: usize, spin: Spin) -> Self {
        let char_w = char_w.max(3);
        let char_h = char_h.max(1);

        // Arc width: explicit value, or auto (~⅓ of bar, min 4 so fade shows).
        let arc_cols = if arc_width > 0 {
            arc_width.min(char_w.saturating_sub(1))
        } else {
            char_w.div_ceil(3).max(4)
        };

        let going_forward = matches!(spin, Spin::Clockwise);
        let anchor = if going_forward {
            0
        } else {
            char_w.saturating_sub(arc_cols)
        };

        Self {
            char_w,
            char_h,
            arc_cols,
            anchor,
            going_forward,
        }
    }

    /// Advance one step, reversing at each edge.
    fn walk(&mut self) {
        let max_anchor = self.char_w.saturating_sub(self.arc_cols);
        if self.going_forward {
            if self.anchor < max_anchor {
                self.anchor += 1;
            } else {
                self.going_forward = false;
            }
        } else if self.anchor > 0 {
            self.anchor -= 1;
        } else {
            self.going_forward = true;
        }
    }

    /// Render the current frame as styled [`Line`]s.
    fn render_lines(
        &self,
        arc_color: Color,
        dim_color: Color,
        fade_width: usize,
        track_byte: u8,
        arc_byte: u8,
        style_chars: Option<(char, char)>,
    ) -> Vec<Line<'static>> {
        let arc_end = self.anchor + self.arc_cols;

        (0..self.char_h)
            .map(|_| {
                let spans: Vec<Span<'static>> = (0..self.char_w)
                    .map(|ci| {
                        let (ch, color) = if let Some((arc_ch, track_ch)) = style_chars {
                            // Symbol style — one char per cell, no braille fade.
                            if ci >= self.anchor && ci < arc_end {
                                (arc_ch, arc_color)
                            } else {
                                (track_ch, dim_color)
                            }
                        } else {
                            // Braille style — density-gradient rendering.
                            if ci >= self.anchor && ci < arc_end {
                                let from_edge = (ci - self.anchor).min(arc_end - 1 - ci);
                                let byte = fade_byte(from_edge, fade_width, arc_byte);
                                let ch =
                                    char::from_u32(0x2800 + u32::from(byte)).unwrap_or('\u{2800}');
                                (ch, arc_color)
                            } else {
                                let ch = char::from_u32(0x2800 + u32::from(track_byte))
                                    .unwrap_or('\u{2800}');
                                (ch, dim_color)
                            }
                        };
                        Span::styled(ch.to_string(), Style::default().fg(color))
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}

// ── Public widget ─────────────────────────────────────────────────────────────

/// A Zed / Claude-style braille loading bar that **bounces** left and right.
///
/// Every character cell in the bar is a braille glyph.  A bright arc window
/// slides across and reverses at each end; the arc edges fade through a
/// density ramp (`⠉ ⠛ ⠿ ⣿`) for a soft comet-glow look.  The dim
/// background uses `⣀` (a two-dot rail) so the full bar extent is visible
/// without competing with the arc.
///
/// # Width
///
/// Leave [`width`](BarSpinner::width) at its default **`0`** to fill
/// the available area automatically (most common usage).  Set it to a fixed
/// positive value if you need a predetermined size.
///
/// # Examples
///
/// ```no_run
/// use ratatui::style::Color;
/// use ratatui::Frame;
/// use ratatui::layout::Rect;
/// use tui_spinner::{BarSpinner, BarTrack, Spin};
///
/// fn draw(frame: &mut Frame, area: Rect, tick: u64) {
///     // Fills the full width of `area` — typical Zed/Claude style.
///     frame.render_widget(
///         BarSpinner::new(tick)
///             .arc_color(Color::Cyan)
///             .dim_color(Color::DarkGray),
///         area,
///     );
/// }
/// ```
///
/// # Field Defaults
///
/// | Field           | Default                     |
/// |-----------------|-----------------------------||
/// | `track`         | [`BarTrack::Rail`]          |
/// | `fade_width`    | `3`                         |
/// | `arc_byte`      | `0xFF` (`⣿`)               |
/// | `bar_style`     | [`BarStyle::Braille`]       |
#[derive(Debug, Clone)]
pub struct BarSpinner<'a> {
    tick: u64,
    /// `0` = fill available area; positive = fixed column count.
    width: usize,
    /// Height in character rows (minimum 1).
    height: usize,
    /// Explicit arc width in character columns (`0` = auto ~⅓ of bar).
    arc_width: usize,
    /// Starting direction before the first bounce.
    spin: Spin,
    /// Ticks held per animation step (higher = slower).
    ticks_per_step: u64,
    /// Colour of the bright arc glyph.
    arc_color: Color,
    /// Colour of the dim background track glyph.
    dim_color: Color,
    /// Background track style (default [`BarTrack::Rail`]).
    track: BarTrack,
    /// Fade ramp width in character columns (default 3; 0 = sharp cutoff).
    fade_width: usize,
    /// Braille byte used for the fully-lit arc centre cells (default `0xFF` = `⣿`).
    arc_byte: u8,
    /// Symbol style for arc and track glyphs (default [`BarStyle::Braille`]).
    bar_style: BarStyle,
    block: Option<Block<'a>>,
    style: Style,
    alignment: Alignment,
}

impl<'a> BarSpinner<'a> {
    // ── Presets ───────────────────────────────────────────────────────────────

    /// **Zed-style** preset — 1 row, cyan arc, subtle Rail track, clockwise.
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    /// let s = BarSpinner::zed(42);
    /// ```
    #[must_use]
    pub fn zed(tick: u64) -> Self {
        Self::new(tick)
            .height(1)
            .arc_color(Color::Cyan)
            .dim_color(Color::DarkGray)
    }

    /// **Claude-style** preset — 2 rows, warm-orange arc, Rail track, clockwise.
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    /// let s = BarSpinner::claude(42);
    /// ```
    #[must_use]
    pub fn claude(tick: u64) -> Self {
        Self::new(tick)
            .height(2)
            .arc_color(Color::Rgb(255, 165, 0))
            .dim_color(Color::DarkGray)
    }

    /// **Minimal** preset — 1 row, white arc, Empty track (arc floats on space).
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    /// let s = BarSpinner::minimal(42);
    /// ```
    #[must_use]
    pub fn minimal(tick: u64) -> Self {
        Self::new(tick)
            .height(1)
            .arc_color(Color::White)
            .dim_color(Color::Black)
            .track(BarTrack::Empty)
    }

    /// **Solid** preset — 1 row, cyan arc, Full track, sharp zero-fade edges.
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    /// let s = BarSpinner::solid(42);
    /// ```
    #[must_use]
    pub fn solid(tick: u64) -> Self {
        Self::new(tick)
            .height(1)
            .arc_color(Color::Cyan)
            .dim_color(Color::DarkGray)
            .track(BarTrack::Full)
            .fade_width(0)
    }

    /// Creates a new [`BarSpinner`] with defaults:
    /// auto-width, 1-row height, clockwise start, cyan arc, dark-gray track,
    /// 1 tick per step, auto arc width.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    ///
    /// let spinner = BarSpinner::new(42);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            tick,
            width: 0,
            height: 1,
            arc_width: 0,
            spin: Spin::Clockwise,
            ticks_per_step: 1,
            arc_color: Color::Cyan,
            dim_color: Color::DarkGray,
            track: BarTrack::Rail,
            fade_width: 3,
            arc_byte: 0xFF,
            bar_style: BarStyle::Braille,
            block: None,
            style: Style::default(),
            alignment: Alignment::Left,
        }
    }

    /// Sets the fixed width in character columns.
    ///
    /// Pass **`0`** (the default) to fill the available area width
    /// automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    ///
    /// let fixed = BarSpinner::new(0).width(24);
    /// let auto  = BarSpinner::new(0).width(0); // fills area
    /// ```
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    /// Sets the height in character rows (minimum 1, default 1).
    ///
    /// Use `1` for a thin Zed-style bar or `2`–`3` for a thicker
    /// Claude-style block.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    ///
    /// let thick = BarSpinner::new(0).height(2);
    /// ```
    #[must_use]
    pub fn height(mut self, h: usize) -> Self {
        self.height = h.max(1);
        self
    }

    /// Sets the arc width in character columns (`0` = auto ~⅓ of bar).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    ///
    /// let narrow = BarSpinner::new(0).arc_width(6);
    /// let wide   = BarSpinner::new(0).arc_width(20);
    /// ```
    #[must_use]
    pub fn arc_width(mut self, w: usize) -> Self {
        self.arc_width = w;
        self
    }

    /// Sets the starting direction before the first bounce
    /// (default: [`Spin::Clockwise`] = starts moving right).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{BarSpinner, Spin};
    ///
    /// let rtl = BarSpinner::new(0).spin(Spin::CounterClockwise);
    /// ```
    #[must_use]
    pub const fn spin(mut self, spin: Spin) -> Self {
        self.spin = spin;
        self
    }

    /// Sets how many ticks each arc position is held (default 1; higher = slower).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    ///
    /// let slow = BarSpinner::new(0).ticks_per_step(3);
    /// ```
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.ticks_per_step = n.max(1);
        self
    }

    /// Sets the colour of the bright arc glyph (default: [`Color::Cyan`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::BarSpinner;
    ///
    /// let spinner = BarSpinner::new(0).arc_color(Color::LightBlue);
    /// ```
    #[must_use]
    pub const fn arc_color(mut self, color: Color) -> Self {
        self.arc_color = color;
        self
    }

    /// Sets the colour of the dim background track (default: [`Color::DarkGray`]).
    ///
    /// Set to the terminal background colour (e.g. [`Color::Black`]) to hide
    /// the track so only the glowing arc is visible.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::BarSpinner;
    ///
    /// // Visible track
    /// let with_track    = BarSpinner::new(0).dim_color(Color::DarkGray);
    /// // Arc floats on empty space
    /// let no_track      = BarSpinner::new(0).dim_color(Color::Black);
    /// ```
    #[must_use]
    pub const fn dim_color(mut self, color: Color) -> Self {
        self.dim_color = color;
        self
    }

    /// Sets the background track style (default [`BarTrack::Rail`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{BarSpinner, BarTrack};
    ///
    /// let solid = BarSpinner::new(0).track(BarTrack::Full);
    /// let float = BarSpinner::new(0).track(BarTrack::Empty);
    /// ```
    #[must_use]
    pub fn track(mut self, track: BarTrack) -> Self {
        self.track = track;
        self
    }

    /// Sets the arc fade-ramp width in character columns (default `3`).
    ///
    /// `0` = sharp cutoff — the arc edge is a hard boundary.
    /// `1`–`3` = progressively softer gradient (default `3` gives `⠉ ⠛ ⠿ ⣿`).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    ///
    /// let sharp = BarSpinner::new(0).fade_width(0);
    /// let soft  = BarSpinner::new(0).fade_width(3); // default
    /// ```
    #[must_use]
    pub fn fade_width(mut self, w: usize) -> Self {
        self.fade_width = w;
        self
    }

    /// Sets the braille byte for the fully-lit arc centre cells (default `0xFF` = `⣿`).
    ///
    /// Use any braille byte to change the arc density.  The fade ramp always
    /// starts from `⠉` and tapers *up to* this value.
    ///
    /// | Example byte | Glyph | Dots |
    /// |---|---|---|
    /// | `0xFF` | `⣿` | 8 — full (default) |
    /// | `0x7F` | `⡿` | 7 |
    /// | `0x3F` | `⠿` | 6 |
    /// | `0x1B` | `⠛` | 4 |
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    ///
    /// let light = BarSpinner::new(0).arc_char(0x3F); // ⠿ lighter arc
    /// ```
    #[must_use]
    pub fn arc_char(mut self, byte: u8) -> Self {
        self.arc_byte = byte;
        self
    }

    /// Sets the glyph style for the arc and background track
    /// (default [`BarStyle::Braille`]).
    ///
    /// Symbol styles (`Block`, `Shade`, `Dot`, `Diamond`, `Square`) use a
    /// single Unicode character for the arc and one for the track.  They
    /// ignore `arc_char`, `track`, and `fade_width`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{BarSpinner, BarStyle};
    ///
    /// let block = BarSpinner::new(0).bar_style(BarStyle::Block);
    /// let dot   = BarSpinner::new(0).bar_style(BarStyle::Dot);
    /// ```
    #[must_use]
    pub fn bar_style(mut self, style: BarStyle) -> Self {
        self.bar_style = style;
        self
    }

    /// Wraps the spinner in a [`Block`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::widgets::Block;
    /// use tui_spinner::BarSpinner;
    ///
    /// let spinner = BarSpinner::new(0)
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

    /// Sets the horizontal alignment of the rendered output (default: left).
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Returns the explicit rendered size `(cols, rows)`, or `None` when the
    /// width is set to auto (`0`).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::BarSpinner;
    ///
    /// assert_eq!(
    ///     BarSpinner::new(0).width(20).height(2).char_size(),
    ///     Some((20, 2))
    /// );
    /// assert_eq!(BarSpinner::new(0).char_size(), None);
    /// ```
    #[must_use]
    pub fn char_size(&self) -> Option<(usize, usize)> {
        if self.width == 0 {
            None
        } else {
            Some((self.width.max(3), self.height.max(1)))
        }
    }

    fn build_lines(&self, actual_width: usize) -> Vec<Line<'static>> {
        let w = actual_width.max(3);
        let mut engine = RectEngine::build(w, self.height, self.arc_width, self.spin);

        #[allow(clippy::cast_possible_truncation)]
        let steps = (self.tick / self.ticks_per_step) as usize;
        for _ in 0..steps {
            engine.walk();
        }

        engine.render_lines(
            self.arc_color,
            self.dim_color,
            self.fade_width,
            self.track.byte(),
            self.arc_byte,
            self.bar_style.chars(),
        )
    }
}

// ── Trait impls ───────────────────────────────────────────────────────────────

impl Styled for BarSpinner<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
}

impl Widget for BarSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &BarSpinner<'_> {
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

        // Resolve width: explicit fixed value, or fill available area.
        let actual_width = if self.width == 0 {
            inner_area.width as usize
        } else {
            self.width
        };

        let lines = self.build_lines(actual_width);
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

    // ── Engine: construction ──────────────────────────────────────────────────

    #[test]
    fn engine_builds_without_panic() {
        for w in 3..=30usize {
            for h in 1..=5usize {
                for spin in [Spin::Clockwise, Spin::CounterClockwise] {
                    let _ = RectEngine::build(w, h, 0, spin);
                }
            }
        }
    }

    #[test]
    fn engine_walk_does_not_panic() {
        let mut e = RectEngine::build(20, 1, 0, Spin::Clockwise);
        for _ in 0..1000 {
            e.walk();
        }
    }

    #[test]
    fn engine_anchor_stays_in_bounds() {
        let mut e = RectEngine::build(20, 1, 0, Spin::Clockwise);
        let max_anchor = e.char_w.saturating_sub(e.arc_cols);
        for _ in 0..500 {
            e.walk();
            assert!(
                e.anchor <= max_anchor,
                "anchor={} exceeds max={max_anchor}",
                e.anchor
            );
        }
    }

    #[test]
    fn engine_bounces_direction() {
        let mut e = RectEngine::build(20, 1, 0, Spin::Clockwise);
        assert!(e.going_forward, "should start going forward (CW)");

        // Walk until direction reverses to backward.
        let mut reversed = false;
        for _ in 0..100 {
            e.walk();
            if !e.going_forward {
                reversed = true;
                break;
            }
        }
        assert!(reversed, "engine never reversed to backward");

        // Walk until it reverses back to forward.
        let mut re_reversed = false;
        for _ in 0..100 {
            e.walk();
            if e.going_forward {
                re_reversed = true;
                break;
            }
        }
        assert!(re_reversed, "engine never reversed back to forward");
    }

    #[test]
    fn cw_starts_at_left_ccw_at_right() {
        let cw = RectEngine::build(20, 1, 0, Spin::Clockwise);
        let ccw = RectEngine::build(20, 1, 0, Spin::CounterClockwise);
        assert_eq!(cw.anchor, 0, "CW should start at column 0");
        assert!(
            ccw.anchor > 0,
            "CCW should start at the right (anchor={})",
            ccw.anchor
        );
        assert!(cw.going_forward);
        assert!(!ccw.going_forward);
    }

    // ── Fade ramp ─────────────────────────────────────────────────────────────

    #[test]
    fn arc_edges_use_fade_bytes() {
        // Build a wide engine so there is a full-density centre.
        let e = RectEngine::build(20, 1, 12, Spin::Clockwise);
        let lines = e.render_lines(Color::Cyan, Color::DarkGray, 3, DIM_BYTE, 0xFF, None);
        assert_eq!(lines.len(), 1);
        let spans = &lines[0].spans;

        // The outermost arc column (index anchor+0) should be FADE[0] = 0x09.
        let outer = char::from_u32(0x2800 + u32::from(FADE[0])).unwrap();
        assert_eq!(
            spans[e.anchor].content.chars().next(),
            Some(outer),
            "outermost arc edge should be FADE[0]"
        );

        // The centre arc column (index anchor + arc_cols/2) should be FADE[3] = 0xFF.
        let centre_idx = e.anchor + e.arc_cols / 2;
        let full = char::from_u32(0x2800 + u32::from(FADE[3])).unwrap();
        assert_eq!(
            spans[centre_idx].content.chars().next(),
            Some(full),
            "arc centre should be full density FADE[3]"
        );
    }

    #[test]
    fn dim_columns_use_dim_byte() {
        let e = RectEngine::build(20, 1, 6, Spin::Clockwise);
        let lines = e.render_lines(Color::Cyan, Color::DarkGray, 3, DIM_BYTE, 0xFF, None);
        let spans = &lines[0].spans;
        let dim_char = char::from_u32(0x2800 + u32::from(DIM_BYTE)).unwrap();
        // Columns before the arc anchor should all be DIM_BYTE.
        for i in 0..e.anchor {
            assert_eq!(
                spans[i].content.chars().next(),
                Some(dim_char),
                "column {i} should be dim"
            );
        }
    }

    // ── Widget output ─────────────────────────────────────────────────────────

    #[test]
    fn build_lines_height_matches() {
        for h in 1..=5usize {
            let lines = BarSpinner::new(0).width(20).height(h).build_lines(20);
            assert_eq!(lines.len(), h, "height={h}");
        }
    }

    #[test]
    fn build_lines_width_matches() {
        let w = 24usize;
        let lines = BarSpinner::new(0).width(w).height(1).build_lines(w);
        assert_eq!(lines[0].spans.len(), w, "each line should have {w} spans");
    }

    #[test]
    fn different_ticks_produce_different_output() {
        let a = BarSpinner::new(0).width(20).height(1).build_lines(20);
        let b = BarSpinner::new(8).width(20).height(1).build_lines(20);
        assert_ne!(a, b, "tick=0 and tick=8 should differ");
    }

    #[test]
    fn cw_and_ccw_differ_at_same_tick() {
        let cw = BarSpinner::new(5)
            .width(20)
            .height(1)
            .spin(Spin::Clockwise)
            .build_lines(20);
        let ccw = BarSpinner::new(5)
            .width(20)
            .height(1)
            .spin(Spin::CounterClockwise)
            .build_lines(20);
        assert_ne!(cw, ccw, "CW and CCW should differ at the same tick");
    }

    #[test]
    fn ticks_per_step_slows_animation() {
        let fast = BarSpinner::new(10)
            .width(20)
            .height(1)
            .ticks_per_step(1)
            .build_lines(20);
        let slow = BarSpinner::new(10)
            .width(20)
            .height(1)
            .ticks_per_step(5)
            .build_lines(20);
        assert_ne!(
            fast, slow,
            "different speeds should produce different output"
        );
    }

    #[test]
    fn arc_width_override_respected() {
        let e = RectEngine::build(20, 1, 7, Spin::Clockwise);
        assert_eq!(e.arc_cols, 7);
    }

    // ── Widget rendering ──────────────────────────────────────────────────────

    #[test]
    fn widget_renders_without_panic() {
        let backend = TestBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(BarSpinner::new(42), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn widget_fixed_width_renders_without_panic() {
        let backend = TestBackend::new(40, 3);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(BarSpinner::new(42).width(24).height(2), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn widget_zero_area_no_panic() {
        let backend = TestBackend::new(0, 0);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(BarSpinner::new(0), frame.area());
            })
            .unwrap();
    }

    #[test]
    fn char_size_fixed_width() {
        let s = BarSpinner::new(0).width(20).height(2);
        assert_eq!(s.char_size(), Some((20, 2)));
    }

    #[test]
    fn char_size_auto_width_is_none() {
        let s = BarSpinner::new(0); // width = 0 = auto
        assert_eq!(s.char_size(), None);
    }

    #[test]
    fn char_size_clamps_minimum() {
        let s = BarSpinner::new(0).width(1).height(0);
        if let Some((w, h)) = s.char_size() {
            assert!(w >= 3, "width clamped to at least 3");
            assert!(h >= 1, "height clamped to at least 1");
        }
    }

    // ── Builder chain ─────────────────────────────────────────────────────────

    #[test]
    fn builder_chain() {
        use ratatui::widgets::Block;
        let s = BarSpinner::new(0)
            .width(24)
            .height(2)
            .arc_width(8)
            .spin(Spin::CounterClockwise)
            .ticks_per_step(3)
            .arc_color(Color::Blue)
            .dim_color(Color::Black)
            .block(Block::bordered())
            .alignment(Alignment::Center);
        assert_eq!(s.width, 24);
        assert_eq!(s.height, 2);
        assert_eq!(s.arc_width, 8);
        assert!(matches!(s.spin, Spin::CounterClockwise));
        assert_eq!(s.ticks_per_step, 3);
        assert_eq!(s.arc_color, Color::Blue);
        assert_eq!(s.dim_color, Color::Black);
    }

    #[test]
    fn arc_char_changes_centre_byte() {
        let e = RectEngine::build(20, 1, 10, Spin::Clockwise);
        // Default arc_byte = 0xFF → centre cell is ⣿
        let lines_default = e.render_lines(Color::Cyan, Color::DarkGray, 3, DIM_BYTE, 0xFF, None);
        // Custom arc_byte = 0x3F → centre cell is ⠿
        let lines_custom = e.render_lines(Color::Cyan, Color::DarkGray, 3, DIM_BYTE, 0x3F, None);
        assert_ne!(
            lines_default, lines_custom,
            "different arc_byte produces different output"
        );
        // The centre span in lines_custom should be ⠿ (U+283F)
        let centre_idx = e.anchor + e.arc_cols / 2;
        let centre_char = lines_custom[0].spans[centre_idx]
            .content
            .chars()
            .next()
            .unwrap();
        assert_eq!(
            centre_char, '\u{283F}',
            "centre cell should be ⠿ when arc_byte=0x3F"
        );
    }

    #[test]
    fn preset_zed_defaults() {
        let s = BarSpinner::zed(0);
        assert_eq!(s.height, 1);
        assert_eq!(s.arc_color, Color::Cyan);
    }

    #[test]
    fn preset_solid_has_full_track_and_zero_fade() {
        let s = BarSpinner::solid(0);
        assert_eq!(s.track, BarTrack::Full);
        assert_eq!(s.fade_width, 0);
    }

    #[test]
    fn track_and_fade_width_builder() {
        let s = BarSpinner::new(0).track(BarTrack::Full).fade_width(0);
        assert_eq!(s.track, BarTrack::Full);
        assert_eq!(s.fade_width, 0);

        // Sharp fade: every arc cell should show full density (FADE[3] = 0xFF).
        let lines = s.width(12).build_lines(12);
        // All spans should be ⣿ (U+28FF) in arc_color OR ⣿ in dim_color (Full track).
        for line in &lines {
            for span in &line.spans {
                let ch = span.content.chars().next().unwrap();
                assert_eq!(ch, '\u{28FF}', "sharp fade + Full track → every cell is ⣿");
            }
        }
    }

    #[test]
    fn bar_style_block_produces_non_braille_chars() {
        let lines = BarSpinner::new(0)
            .width(20)
            .bar_style(BarStyle::Block)
            .build_lines(20);
        // Every character should be either █ or ░
        for line in &lines {
            for span in &line.spans {
                let ch = span.content.chars().next().unwrap();
                assert!(
                    ch == '█' || ch == '░',
                    "Block style: unexpected char U+{:04X}",
                    ch as u32
                );
            }
        }
    }

    #[test]
    fn all_non_braille_styles_have_char_pairs() {
        let styles = [
            BarStyle::Block,
            BarStyle::Shade,
            BarStyle::Dot,
            BarStyle::Diamond,
            BarStyle::Square,
            BarStyle::Star,
            BarStyle::Heart,
            BarStyle::Arrow,
            BarStyle::Circle,
            BarStyle::Spark,
            BarStyle::Cross,
            BarStyle::Progress,
            BarStyle::Thick,
            BarStyle::Wave,
            BarStyle::Pip,
        ];
        for style in styles {
            assert!(style.chars().is_some(), "{style:?} should have a char pair");
        }
        assert!(BarStyle::Braille.chars().is_none());
    }

    #[test]
    fn bar_style_builder() {
        let s = BarSpinner::new(0).bar_style(BarStyle::Dot);
        assert_eq!(s.bar_style, BarStyle::Dot);
    }
}
