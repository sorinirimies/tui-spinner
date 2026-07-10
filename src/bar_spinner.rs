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
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
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

// ── Motion mode ───────────────────────────────────────────────────────────────

/// Controls how the arc behaves when it reaches the edge of the bar.
///
/// | Variant    | Behaviour |
/// |------------|----------------------------------------------------------------------|
/// | `Bounce`   | Reverses at each edge — classic ping-pong (default)                   |
/// | `Loop`     | Wraps around: when the arc exits one edge it re-enters the other     |
/// | `Squeeze`  | Two arcs converge from both edges toward the centre then bounce back |
/// | `Radiate`  | Two arcs radiate outward from the centre and wrap back continuously  |
///
/// Combined with [`Spin`], `Loop` produces a continuous sweep:
/// - `Spin::Clockwise` + `Loop` → sweeps left → right endlessly
/// - `Spin::CounterClockwise` + `Loop` → sweeps right → left endlessly
///
/// # Examples
///
/// ```
/// use tui_spinner::{BarSpinner, BarMotion, Spin};
///
/// // Default ping-pong
/// let bounce = BarSpinner::new(0).motion(BarMotion::Bounce);
///
/// // Continuous left-to-right sweep
/// let sweep = BarSpinner::new(0).spin(Spin::Clockwise).motion(BarMotion::Loop);
///
/// // Converge from both edges
/// let squeeze = BarSpinner::new(0).motion(BarMotion::Squeeze);
///
/// // Radiate outward from centre continuously
/// let radiate = BarSpinner::new(0).motion(BarMotion::Radiate);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarMotion {
    /// Reverse at each edge — ping-pong (default).
    #[default]
    Bounce,
    /// Wrap around: exit one edge and re-enter from the other.
    Loop,
    /// Two arcs converge from both edges toward the centre, then bounce back.
    Squeeze,
    /// Two arcs radiate outward from the centre and wrap back continuously.
    Radiate,
}

// ── Orientation ───────────────────────────────────────────────────────────────

/// Controls whether the bar slides horizontally or vertically.
///
/// | Variant      | Arc motion        |
/// |--------------|-------------------|
/// | `Horizontal` | left ↔ right (default) |
/// | `Vertical`   | top ↕ bottom      |
///
/// # Examples
///
/// ```
/// use tui_spinner::{BarSpinner, BarOrientation};
///
/// let h = BarSpinner::new(0).orientation(BarOrientation::Horizontal);
/// let v = BarSpinner::new(0).orientation(BarOrientation::Vertical);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarOrientation {
    /// Arc bounces / loops left ↔ right (default).
    #[default]
    Horizontal,
    /// Arc bounces / loops top ↕ bottom.
    Vertical,
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
    motion: BarMotion,
}

impl RectEngine {
    fn build(
        char_w: usize,
        char_h: usize,
        arc_width: usize,
        spin: Spin,
        motion: BarMotion,
    ) -> Self {
        let char_w = char_w.max(3);
        let char_h = char_h.max(1);

        // Arc width: explicit value, or auto (~⅓ of bar, min 4 so fade shows).
        let arc_cols = if arc_width > 0 {
            arc_width.min(char_w.saturating_sub(1))
        } else {
            char_w.div_ceil(3).max(4)
        };

        let going_forward = match motion {
            // Squeeze always starts moving inward (forward = converging).
            // Radiate always starts moving outward.
            BarMotion::Squeeze | BarMotion::Radiate => true,
            _ => matches!(spin, Spin::Clockwise),
        };
        let anchor = match motion {
            BarMotion::Squeeze | BarMotion::Radiate => 0, // phase 0 = arcs at centre/edges
            _ => {
                if going_forward {
                    0
                } else {
                    char_w.saturating_sub(arc_cols)
                }
            }
        };

        Self {
            char_w,
            char_h,
            arc_cols,
            anchor,
            going_forward,
            motion,
        }
    }

    /// Advance one step, reversing or wrapping at each edge.
    fn walk(&mut self) {
        match self.motion {
            BarMotion::Bounce => {
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
            BarMotion::Loop => {
                // Anchor travels the full 0..char_w range modularly.
                if self.going_forward {
                    self.anchor = (self.anchor + 1) % self.char_w;
                } else {
                    self.anchor = (self.anchor + self.char_w - 1) % self.char_w;
                }
            }
            BarMotion::Squeeze => {
                // `anchor` represents how far each arc has moved inward from
                // its respective edge.  Max inward = centre meeting point.
                let half = self.char_w.saturating_sub(self.arc_cols) / 2;
                if self.going_forward {
                    if self.anchor < half {
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
            BarMotion::Radiate => {
                // `anchor` represents how far each arc has moved outward from
                // the centre.  Wraps back to 0 (centre) when it reaches the edge.
                let half = self.char_w.saturating_sub(self.arc_cols) / 2;
                self.anchor = (self.anchor + 1) % (half + 1);
            }
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
        let char_w = self.char_w;
        let arc_cols = self.arc_cols;

        (0..self.char_h)
            .map(|_| {
                let spans: Vec<Span<'static>> = (0..char_w)
                    .map(|ci| {
                        // Determine whether this column is inside the arc and,
                        // if so, its distance from the nearer arc edge.
                        let (in_arc, from_edge) = match self.motion {
                            BarMotion::Bounce => {
                                let arc_end = self.anchor + arc_cols;
                                if ci >= self.anchor && ci < arc_end {
                                    let fe = (ci - self.anchor).min(arc_end - 1 - ci);
                                    (true, fe)
                                } else {
                                    (false, 0)
                                }
                            }
                            BarMotion::Loop => {
                                // Modular offset: how far past `anchor` is `ci`?
                                let offset = (ci + char_w - self.anchor) % char_w;
                                if offset < arc_cols {
                                    let fe = offset.min(arc_cols - 1 - offset);
                                    (true, fe)
                                } else {
                                    (false, 0)
                                }
                            }
                            BarMotion::Squeeze => {
                                // Two arcs: one from left, one from right.
                                // Left arc: starts at `anchor`, width `arc_cols`.
                                // Right arc: mirror from right edge.
                                let left_start = self.anchor;
                                let left_end = left_start + arc_cols;
                                let right_end = char_w.saturating_sub(self.anchor);
                                let right_start = right_end.saturating_sub(arc_cols);

                                if ci >= left_start && ci < left_end {
                                    let fe = (ci - left_start).min(left_end - 1 - ci);
                                    (true, fe)
                                } else if ci >= right_start && ci < right_end {
                                    let fe = (ci - right_start).min(right_end - 1 - ci);
                                    (true, fe)
                                } else {
                                    (false, 0)
                                }
                            }
                            BarMotion::Radiate => {
                                // Two arcs emanating from the centre outward.
                                // `anchor` = how far each arc has moved from centre.
                                let centre = char_w / 2;
                                // Right arc: centre moves rightward by `anchor`.
                                let right_start = centre + self.anchor;
                                let right_end = right_start + arc_cols;
                                // Left arc: centre moves leftward by `anchor`.
                                let left_end = centre.saturating_sub(self.anchor);
                                let left_start = left_end.saturating_sub(arc_cols);

                                if ci >= right_start && ci < right_end.min(char_w) {
                                    let fe = (ci - right_start).min(right_end.min(char_w) - 1 - ci);
                                    (true, fe)
                                } else if ci >= left_start && ci < left_end {
                                    let fe = (ci - left_start).min(left_end - 1 - ci);
                                    (true, fe)
                                } else {
                                    (false, 0)
                                }
                            }
                        };

                        let (ch, color) = if let Some((arc_ch, track_ch)) = style_chars {
                            if in_arc {
                                (arc_ch, arc_color)
                            } else {
                                (track_ch, dim_color)
                            }
                        } else if in_arc {
                            let byte = fade_byte(from_edge, fade_width, arc_byte);
                            let ch = char::from_u32(0x2800 + u32::from(byte)).unwrap_or('\u{2800}');
                            (ch, arc_color)
                        } else {
                            let ch = char::from_u32(0x2800 + u32::from(track_byte))
                                .unwrap_or('\u{2800}');
                            (ch, dim_color)
                        };

                        Span::styled(ch.to_string(), Style::default().fg(color))
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}

// ── Vertical engine ───────────────────────────────────────────────────────────

/// Internal engine for vertical [`BarSpinner`] orientation.
///
/// Mirrors [`RectEngine`] but animates in the row dimension instead of columns.
struct VertRectEngine {
    char_w: usize,
    char_h: usize,
    arc_rows: usize,
    anchor: usize,
    going_forward: bool,
    motion: BarMotion,
}

impl VertRectEngine {
    fn build(
        char_w: usize,
        char_h: usize,
        arc_height: usize,
        spin: Spin,
        motion: BarMotion,
    ) -> Self {
        let char_w = char_w.max(1);
        let char_h = char_h.max(3);

        let arc_rows = if arc_height > 0 {
            arc_height.min(char_h.saturating_sub(1))
        } else {
            char_h.div_ceil(3).max(2)
        };

        let going_forward = match motion {
            BarMotion::Squeeze | BarMotion::Radiate => true,
            _ => matches!(spin, Spin::Clockwise),
        };
        let anchor = match motion {
            BarMotion::Squeeze | BarMotion::Radiate => 0,
            _ => {
                if going_forward {
                    0
                } else {
                    char_h.saturating_sub(arc_rows)
                }
            }
        };

        Self {
            char_w,
            char_h,
            arc_rows,
            anchor,
            going_forward,
            motion,
        }
    }

    fn walk(&mut self) {
        match self.motion {
            BarMotion::Bounce => {
                let max_anchor = self.char_h.saturating_sub(self.arc_rows);
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
            BarMotion::Loop => {
                if self.going_forward {
                    self.anchor = (self.anchor + 1) % self.char_h;
                } else {
                    self.anchor = (self.anchor + self.char_h - 1) % self.char_h;
                }
            }
            BarMotion::Squeeze => {
                let half = self.char_h.saturating_sub(self.arc_rows) / 2;
                if self.going_forward {
                    if self.anchor < half {
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
            BarMotion::Radiate => {
                // `anchor` = how far each arc has moved outward from centre.
                // Wraps back to 0 (centre) when it reaches the edge.
                let half = self.char_h.saturating_sub(self.arc_rows) / 2;
                self.anchor = (self.anchor + 1) % (half + 1);
            }
        }
    }

    fn render_lines(
        &self,
        arc_color: Color,
        dim_color: Color,
        track_byte: u8,
        arc_byte: u8,
        style_chars: Option<(char, char)>,
    ) -> Vec<Line<'static>> {
        let char_h = self.char_h;
        let arc_rows = self.arc_rows;

        (0..char_h)
            .map(|row| {
                let in_arc = match self.motion {
                    BarMotion::Bounce => row >= self.anchor && row < self.anchor + arc_rows,
                    BarMotion::Loop => (row + char_h - self.anchor) % char_h < arc_rows,
                    BarMotion::Squeeze => {
                        // Two arcs: one from top, one from bottom.
                        let top_start = self.anchor;
                        let top_end = top_start + arc_rows;
                        let bot_end = char_h.saturating_sub(self.anchor);
                        let bot_start = bot_end.saturating_sub(arc_rows);
                        (row >= top_start && row < top_end) || (row >= bot_start && row < bot_end)
                    }
                    BarMotion::Radiate => {
                        // Two arcs emanating from centre outward.
                        let centre = char_h / 2;
                        // Downward arc: starts at centre, moves down by `anchor`.
                        let down_start = centre + self.anchor;
                        let down_end = down_start + arc_rows;
                        // Upward arc: starts at centre, moves up by `anchor`.
                        let up_end = centre.saturating_sub(self.anchor);
                        let up_start = up_end.saturating_sub(arc_rows);
                        (row >= down_start && row < down_end.min(char_h))
                            || (row >= up_start && row < up_end)
                    }
                };

                let (ch, color) = if let Some((arc_ch, track_ch)) = style_chars {
                    if in_arc {
                        (arc_ch, arc_color)
                    } else {
                        (track_ch, dim_color)
                    }
                } else if in_arc {
                    (
                        char::from_u32(0x2800 + u32::from(arc_byte)).unwrap_or('\u{2800}'),
                        arc_color,
                    )
                } else {
                    (
                        char::from_u32(0x2800 + u32::from(track_byte)).unwrap_or('\u{2800}'),
                        dim_color,
                    )
                };

                let spans: Vec<Span<'static>> = (0..self.char_w)
                    .map(|_| Span::styled(ch.to_string(), Style::default().fg(color)))
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
/// | `motion`        | [`BarMotion::Bounce`]       |
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
    /// Arc motion mode (default [`BarMotion::Bounce`]).
    motion: BarMotion,
    /// Bar orientation: horizontal (default) or vertical.
    orientation: BarOrientation,
    /// Cross-axis thickness override (`0` = use `height` for horizontal / `width` for vertical).
    thickness: usize,
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
            motion: BarMotion::Bounce,
            orientation: BarOrientation::Horizontal,
            thickness: 0,
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

    /// Sets both `arc_color` and `dim_color` in one call.
    ///
    /// Equivalent to `.arc_color(arc).dim_color(dim)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::BarSpinner;
    ///
    /// let s = BarSpinner::new(0).with_colors(Color::Cyan, Color::DarkGray);
    /// ```
    #[must_use]
    pub fn with_colors(mut self, arc_color: Color, dim_color: Color) -> Self {
        self.arc_color = arc_color;
        self.dim_color = dim_color;
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

    /// Sets the arc motion mode (default [`BarMotion::Bounce`]).
    ///
    /// - [`BarMotion::Bounce`] — reverses at each edge (ping-pong).
    /// - [`BarMotion::Loop`] — wraps around; use with [`Spin`] to set the sweep direction.
    /// - [`BarMotion::Squeeze`] — two arcs converge from both edges toward the centre then bounce back.
    /// - [`BarMotion::Radiate`] — two arcs radiate outward from the centre and wrap back continuously.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{BarSpinner, BarMotion, Spin};
    ///
    /// // Continuous left-to-right sweep
    /// let sweep = BarSpinner::new(0)
    ///     .spin(Spin::Clockwise)
    ///     .motion(BarMotion::Loop);
    ///
    /// // Converge from both edges
    /// let squeeze = BarSpinner::new(0).motion(BarMotion::Squeeze);
    ///
    /// // Radiate outward from centre
    /// let radiate = BarSpinner::new(0).motion(BarMotion::Radiate);
    /// ```
    #[must_use]
    pub fn motion(mut self, motion: BarMotion) -> Self {
        self.motion = motion;
        self
    }

    /// Sets the bar orientation (default [`BarOrientation::Horizontal`]).
    ///
    /// - [`BarOrientation::Horizontal`] — arc slides left ↔ right (default).
    /// - [`BarOrientation::Vertical`]   — arc slides top ↕ bottom.
    ///
    /// For vertical bars, `width` controls the number of character columns
    /// and `height` (or the available area height) controls the bar length.
    /// `arc_width` controls the height of the bright arc in rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{BarSpinner, BarOrientation};
    ///
    /// let v = BarSpinner::new(0)
    ///     .orientation(BarOrientation::Vertical)
    ///     .width(3);
    /// ```
    #[must_use]
    pub fn orientation(mut self, orientation: BarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the cross-axis **thickness** (default `0` = use existing `height`/`width`).
    ///
    /// This is an orientation-aware convenience:
    /// - **Horizontal** bars → sets the row count (same as `.height()`).
    /// - **Vertical** bars   → sets the column count (same as `.width()`).
    ///
    /// Pass `0` to fall back to the per-axis defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{BarSpinner, BarOrientation};
    ///
    /// // Thick horizontal bar (3 rows):
    /// let h = BarSpinner::new(0).thickness(3);
    ///
    /// // Thick vertical bar (3 columns):
    /// let v = BarSpinner::new(0)
    ///     .orientation(BarOrientation::Vertical)
    ///     .thickness(3);
    /// ```
    #[must_use]
    pub fn thickness(mut self, n: usize) -> Self {
        self.thickness = n;
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

    /// Renders the current frame as a `Vec<Line>`, one [`Line`] per row,
    /// laid out for the given character `width` and `height`.
    ///
    /// Unlike the other spinners a bar has no intrinsic size when width is set
    /// to auto (`0`), so the target dimensions must be supplied explicitly.
    /// This lets the spinner be embedded in other widgets that accept
    /// [`Line`]s or [`Text`], such as a table
    /// [`Cell`](ratatui::widgets::Cell):
    ///
    /// ```
    /// use ratatui::widgets::Cell;
    /// use tui_spinner::BarSpinner;
    ///
    /// let spinner = BarSpinner::new(0);
    /// let _cell = Cell::from(spinner.to_lines(20, 1));
    /// ```
    #[must_use]
    pub fn to_lines(&self, width: usize, height: usize) -> Vec<Line<'static>> {
        self.build_lines(width, height)
    }

    /// Renders the current frame as a [`Text`] value laid out for the given
    /// character `width` and `height`, for embedding in widgets whose content
    /// is a [`Text`] (e.g. a table [`Cell`](ratatui::widgets::Cell) or
    /// [`Paragraph`](ratatui::widgets::Paragraph)).
    #[must_use]
    pub fn to_text(&self, width: usize, height: usize) -> Text<'static> {
        Text::from(self.build_lines(width, height))
    }

    fn build_lines(&self, actual_width: usize, actual_height: usize) -> Vec<Line<'static>> {
        #[allow(clippy::cast_possible_truncation)]
        let steps = (self.tick / self.ticks_per_step) as usize;

        match self.orientation {
            BarOrientation::Horizontal => {
                let w = actual_width.max(3);
                // thickness overrides row count for horizontal bars.
                let h = if self.thickness > 0 {
                    self.thickness
                } else {
                    self.height
                };
                // Disable fade for multi-row bars: the graduated braille dots
                // create ugly stepped/diagonal edges when stacked vertically.
                // Single-row bars keep the smooth gradient.
                let fade = if h > 1 { 0 } else { self.fade_width };
                let mut engine = RectEngine::build(w, h, self.arc_width, self.spin, self.motion);
                for _ in 0..steps {
                    engine.walk();
                }
                engine.render_lines(
                    self.arc_color,
                    self.dim_color,
                    fade,
                    self.track.byte(),
                    self.arc_byte,
                    self.bar_style.chars(),
                )
            }
            BarOrientation::Vertical => {
                // For vertical bars the width field controls the column count
                // and the available area height drives the bar length.
                // thickness overrides column count for vertical bars.
                let char_w = if self.thickness > 0 {
                    self.thickness
                } else if self.width == 0 {
                    actual_width
                } else {
                    self.width
                }
                .max(1);
                let char_h = actual_height.max(3);
                let mut engine =
                    VertRectEngine::build(char_w, char_h, self.arc_width, self.spin, self.motion);
                for _ in 0..steps {
                    engine.walk();
                }
                engine.render_lines(
                    self.arc_color,
                    self.dim_color,
                    self.track.byte(),
                    self.arc_byte,
                    self.bar_style.chars(),
                )
            }
        }
    }
}

// ── Trait impls ───────────────────────────────────────────────────────────────

impl_styled_for!(BarSpinner<'_>);

impl_widget_via_ref!(BarSpinner<'_>);

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

        // Horizontal: width drives bar length; height is row thickness.
        // Vertical:   height drives bar length; width is column thickness.
        let actual_width = if self.width == 0 {
            inner_area.width as usize
        } else {
            self.width
        };
        let actual_height = inner_area.height as usize;

        let lines = self.build_lines(actual_width, actual_height);
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
                    let _ = RectEngine::build(w, h, 0, spin, BarMotion::Bounce);
                }
            }
        }
    }

    #[test]
    fn engine_walk_does_not_panic() {
        let mut e = RectEngine::build(20, 1, 0, Spin::Clockwise, BarMotion::Bounce);
        for _ in 0..1000 {
            e.walk();
        }
    }

    #[test]
    fn engine_anchor_stays_in_bounds() {
        let mut e = RectEngine::build(20, 1, 0, Spin::Clockwise, BarMotion::Bounce);
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
        let mut e = RectEngine::build(20, 1, 0, Spin::Clockwise, BarMotion::Bounce);
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
        let cw = RectEngine::build(20, 1, 0, Spin::Clockwise, BarMotion::Bounce);
        let ccw = RectEngine::build(20, 1, 0, Spin::CounterClockwise, BarMotion::Bounce);
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
        let e = RectEngine::build(20, 1, 12, Spin::Clockwise, BarMotion::Bounce);
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
        let e = RectEngine::build(20, 1, 6, Spin::Clockwise, BarMotion::Bounce);
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
            let lines = BarSpinner::new(0).width(20).height(h).build_lines(20, 10);
            assert_eq!(lines.len(), h, "height={h}");
        }
    }

    #[test]
    fn build_lines_width_matches() {
        let w = 24usize;
        let lines = BarSpinner::new(0).width(w).height(1).build_lines(w, 10);
        assert_eq!(lines[0].spans.len(), w, "each line should have {w} spans");
    }

    #[test]
    fn different_ticks_produce_different_output() {
        let a = BarSpinner::new(0).width(20).height(1).build_lines(20, 10);
        let b = BarSpinner::new(8).width(20).height(1).build_lines(20, 10);
        assert_ne!(a, b, "tick=0 and tick=8 should differ");
    }

    #[test]
    fn cw_and_ccw_differ_at_same_tick() {
        let cw = BarSpinner::new(5)
            .width(20)
            .height(1)
            .spin(Spin::Clockwise)
            .build_lines(20, 10);
        let ccw = BarSpinner::new(5)
            .width(20)
            .height(1)
            .spin(Spin::CounterClockwise)
            .build_lines(20, 10);
        assert_ne!(cw, ccw, "CW and CCW should differ at the same tick");
    }

    #[test]
    fn ticks_per_step_slows_animation() {
        let fast = BarSpinner::new(10)
            .width(20)
            .height(1)
            .ticks_per_step(1)
            .build_lines(20, 10);
        let slow = BarSpinner::new(10)
            .width(20)
            .height(1)
            .ticks_per_step(5)
            .build_lines(20, 10);
        assert_ne!(
            fast, slow,
            "different speeds should produce different output"
        );
    }

    #[test]
    fn arc_width_override_respected() {
        let e = RectEngine::build(20, 1, 7, Spin::Clockwise, BarMotion::Bounce);
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
        let e = RectEngine::build(20, 1, 10, Spin::Clockwise, BarMotion::Bounce);
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
        let lines = s.width(12).build_lines(12, 10);
        // All spans should be ⣿ (U+28FF) in arc_color OR ⣿ in dim_color (Full track).
        for line in &lines {
            for span in &line.spans {
                let ch = span.content.chars().next().unwrap();
                assert_eq!(ch, '\u{28FF}', "sharp fade + Full track → every cell is ⣿");
            }
        }
    }

    // ── with_colors convenience ────────────────────────────────────────────────

    #[test]
    fn with_colors_sets_both() {
        use ratatui::style::Color;
        let s = BarSpinner::new(0).with_colors(Color::Red, Color::Blue);
        assert_eq!(s.arc_color, Color::Red);
        assert_eq!(s.dim_color, Color::Blue);
    }

    // ── Arc width clamping ────────────────────────────────────────────────────

    #[test]
    fn arc_width_larger_than_bar_is_clamped() {
        // arc_width > char_w should be clamped to char_w - 1 inside the engine.
        let e = RectEngine::build(10, 1, 99, Spin::Clockwise, BarMotion::Bounce);
        assert!(e.arc_cols < e.char_w, "arc_cols must be < char_w");
    }

    #[test]
    fn arc_width_zero_uses_auto() {
        let e = RectEngine::build(30, 1, 0, Spin::Clockwise, BarMotion::Bounce);
        assert!(e.arc_cols >= 4, "auto arc should be at least 4 cols");
        assert!(e.arc_cols < e.char_w);
    }

    // ── Loop wraps correctly across the full range ────────────────────────────

    #[test]
    fn loop_all_columns_lit_over_full_cycle() {
        let width = 12usize;
        let arc_cols = 4usize;
        let mut e = RectEngine::build(width, 1, arc_cols, Spin::Clockwise, BarMotion::Loop);
        let mut ever_lit = vec![false; width];

        // Walk one full cycle (char_w steps) and record which columns are lit.
        for _ in 0..width {
            let lines = e.render_lines(
                ratatui::style::Color::Cyan,
                ratatui::style::Color::DarkGray,
                3,
                DIM_BYTE,
                0xFF,
                None,
            );
            for (ci, span) in lines[0].spans.iter().enumerate() {
                if span.style.fg == Some(ratatui::style::Color::Cyan) {
                    ever_lit[ci] = true;
                }
            }
            e.walk();
        }

        // Every column should have been lit at least once.
        for (ci, &lit) in ever_lit.iter().enumerate() {
            assert!(lit, "column {ci} was never lit during a full Loop cycle");
        }
    }

    // ── BarStyle produces expected characters ─────────────────────────────────

    #[test]
    fn non_braille_style_chars_match_declaration() {
        for (style, expected_arc, expected_track) in [
            (BarStyle::Block, '\u{2588}', '\u{2591}'),
            (BarStyle::Dot, '\u{25CF}', '\u{00B7}'),
            (BarStyle::Star, '\u{2605}', '\u{2606}'),
            (BarStyle::Progress, '\u{25B0}', '\u{25B1}'),
        ] {
            let Some((arc, track)) = style.chars() else {
                panic!("{style:?} should have char pair");
            };
            assert_eq!(arc, expected_arc, "{style:?} arc char mismatch");
            assert_eq!(track, expected_track, "{style:?} track char mismatch");
        }
    }

    #[test]
    fn bar_style_block_produces_non_braille_chars() {
        let lines = BarSpinner::new(0)
            .width(20)
            .bar_style(BarStyle::Block)
            .build_lines(20, 10);
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

    // ── BarMotion ──────────────────────────────────────────────────────────────────

    #[test]
    fn loop_motion_wraps_at_right_edge() {
        let mut e = RectEngine::build(20, 1, 4, Spin::Clockwise, BarMotion::Loop);
        // Loop anchor travels 0..char_w modularly (not 0..char_w-arc_cols).
        let last = e.char_w - 1;
        for _ in 0..200 {
            if e.anchor == last {
                break;
            }
            e.walk();
        }
        assert_eq!(e.anchor, last, "anchor should reach char_w-1");
        e.walk();
        assert_eq!(e.anchor, 0, "Loop CW should wrap to 0");
        assert!(e.going_forward, "Loop must not flip direction");
    }

    #[test]
    fn loop_motion_wraps_at_left_edge_ccw() {
        let mut e = RectEngine::build(20, 1, 4, Spin::CounterClockwise, BarMotion::Loop);
        // CCW starts at char_w - arc_cols; walk it to 0.
        for _ in 0..200 {
            if e.anchor == 0 {
                break;
            }
            e.walk();
        }
        assert_eq!(e.anchor, 0);
        e.walk();
        assert_eq!(e.anchor, e.char_w - 1, "CCW Loop should wrap to char_w-1");
        assert!(!e.going_forward);
    }

    #[test]
    fn bounce_still_reverses() {
        let mut e = RectEngine::build(20, 1, 4, Spin::Clockwise, BarMotion::Bounce);
        let max = e.char_w - e.arc_cols;
        for _ in 0..100 {
            if e.anchor == max {
                break;
            }
            e.walk();
        }
        e.walk();
        assert!(!e.going_forward, "Bounce must reverse at max_anchor");
    }

    #[test]
    fn motion_builder() {
        let s = BarSpinner::new(0).motion(BarMotion::Loop);
        assert_eq!(s.motion, BarMotion::Loop);
    }

    #[test]
    fn vertical_orientation_builder() {
        let s = BarSpinner::new(0).orientation(BarOrientation::Vertical);
        assert_eq!(s.orientation, BarOrientation::Vertical);
    }

    #[test]
    fn vertical_renders_correct_row_count() {
        let h = 12usize;
        let lines = BarSpinner::new(0)
            .orientation(BarOrientation::Vertical)
            .width(3)
            .build_lines(3, h);
        assert_eq!(lines.len(), h, "vertical bar should produce {h} lines");
    }

    #[test]
    fn vertical_renders_correct_col_count() {
        let w = 4usize;
        let lines = BarSpinner::new(0)
            .orientation(BarOrientation::Vertical)
            .width(w)
            .build_lines(w, 10);
        for line in &lines {
            assert_eq!(line.spans.len(), w, "each row should have {w} spans");
        }
    }

    #[test]
    fn vertical_loop_does_not_panic() {
        let lines = BarSpinner::new(99)
            .orientation(BarOrientation::Vertical)
            .motion(BarMotion::Loop)
            .width(2)
            .build_lines(2, 15);
        assert_eq!(lines.len(), 15);
    }

    #[test]
    fn to_lines_matches_build_lines() {
        let s = BarSpinner::new(3);
        assert_eq!(s.to_lines(20, 1), s.build_lines(20, 1));
        assert_eq!(s.to_text(20, 2).lines.len(), s.build_lines(20, 2).len());
    }
}
