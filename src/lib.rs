//! # tui-spinner
//!
//! Customizable spinner widgets for [Ratatui](https://github.com/ratatui/ratatui) TUI applications.
//!
//! ## Widgets
//!
//! - **[`LinearSpinner`]** — A spinner that animates along a straight axis:
//!   - **[`Direction::Horizontal`]** — a window of lit symbols scrolls left-to-right
//!     across a row (classic ellipsis effect).
//!   - **[`Direction::Vertical`]** — a single lit symbol bounces up and down a column
//!     (the "Zed / Copilot" activity indicator pattern).
//!
//!   The symbol set is controlled by [`LinearStyle`]: [`LinearStyle::Classic`],
//!   [`LinearStyle::Square`], [`LinearStyle::Diamond`], [`LinearStyle::Bar`],
//!   [`LinearStyle::Braille`], and [`LinearStyle::Arrow`].
//!
//! - **[`RectSpinner`]** — A comet-like braille-dot arc that travels around the
//!   perimeter of a rectangle.  The dot grid is sized so the rendered output is
//!   always a **square** character-cell region.
//!
//!   The shape is controlled by [`RectShape`]:
//!   - [`RectShape::Square`] — square character-cell output; the argument is the
//!     arc thickness / size parameter.
//!   - [`RectShape::Narrow`] — a 1-character-wide Zed sidebar style arc.
//!
//!   The spin direction is controlled by [`Spin`]:
//!   - [`Spin::Clockwise`] — arc travels clockwise (default).
//!   - [`Spin::CounterClockwise`] — arc travels counter-clockwise.
//!
//!   The centre fill is controlled by [`Centre`]:
//!   - [`Centre::Filled`] — a solid interior block that alternates colour as
//!     the arc passes the centre column boundary.
//!   - [`Centre::Empty`] — no interior fill; only the moving arc is visible.
//!
//!   The rendering style is controlled by [`RectStyle`]:
//!   - [`RectStyle::Arc`]     — exact braille dot-pattern (default)
//!   - [`RectStyle::Dense`]   — solid `⣿` arc cells
//!   - [`RectStyle::Shade`]   — `█` / `░` block characters
//!   - [`RectStyle::Outline`] — `◉` / `○` circle symbols
//!   - [`RectStyle::Dot`]     — `•` / `·` bullet / middle dot
//!   - [`RectStyle::Star`]    — `★` / `☆` filled / open star
//!   - [`RectStyle::Diamond`] — `◆` / `◇` filled / open diamond
//!   - [`RectStyle::Cross`]   — `╋` / `┼` heavy / light plus
//!   - [`RectStyle::Fade`]    — `█`/`▓`/`▒` by braille bit density
//!   - [`RectStyle::Pixel`]   — `▪` / `▫` small filled / open square
//!
//! - **[`CircleSpinner`]** — A comet-like arc rotating around a **circular**
//!   braille-dot ring.  The perimeter is computed with the midpoint circle
//!   algorithm at 1:1 dot pitch (producing a visually round circle after
//!   braille packing), sorted clockwise, and the head/tail indices step through
//!   it.  Supports [`Spin::Clockwise`] and [`Spin::CounterClockwise`].
//!
//!   Configured with `.radius(n)`, `.spin()`, `.arc_color()`, and `.dim_color()`.
//!
//! - **[`SquareSpinner`]** — Legacy alias kept for backward compatibility.
//!   Delegates to [`RectSpinner`] with [`RectShape::Square`].
//!   Prefer [`RectSpinner`] for new code.
//!
//! ## Quick start
//!
//! ```no_run
//! use ratatui::style::Color;
//! use tui_spinner::{
//!     Centre, CircleSpinner, Direction, LinearSpinner, LinearStyle,
//!     RectShape, RectSpinner, RectStyle, Spin,
//! };
//!
//! // Horizontal ellipsis — default
//! let h = LinearSpinner::new(42);
//!
//! // Vertical bounce with diamond symbols
//! let v = LinearSpinner::new(42)
//!     .direction(Direction::Vertical)
//!     .linear_style(LinearStyle::Diamond)
//!     .active_color(Color::Cyan);
//!
//! // Classic filled square arc, clockwise
//! let sq = RectSpinner::new(42)
//!     .shape(RectShape::Square(2))
//!     .render_style(RectStyle::Arc)
//!     .outer_color(Color::Cyan)
//!     .inner_color(Color::DarkGray)
//!     .centre(Centre::Filled);
//!
//! // Counter-clockwise hollow square
//! let ccw = RectSpinner::new(42)
//!     .shape(RectShape::Square(3))
//!     .spin(Spin::CounterClockwise)
//!     .render_style(RectStyle::Dense)
//!     .outer_color(Color::Green)
//!     .centre(Centre::Empty);
//!
//! // Narrow (Zed-style) 1-char-wide sidebar arc
//! let narrow = RectSpinner::new(42)
//!     .shape(RectShape::Narrow(10))
//!     .outer_color(Color::Green);
//!
//! // Circle spinner — counter-clockwise
//! let circle = CircleSpinner::new(42)
//!     .radius(5)
//!     .spin(Spin::CounterClockwise)
//!     .arc_color(Color::Cyan)
//!     .dim_color(Color::DarkGray);
//! ```
//!
//! ## Integration pattern
//!
//! All widgets are **stateless** — they only need a monotonically increasing
//! `tick: u64` counter (incremented once per render frame).  No mutable widget
//! state is required.
//!
//! ```no_run
//! use ratatui::Frame;
//! use ratatui::layout::Rect;
//! use tui_spinner::{Direction, LinearSpinner};
//!
//! struct App { tick: u64 }
//!
//! fn draw(frame: &mut Frame, area: Rect, app: &App) {
//!     frame.render_widget(
//!         LinearSpinner::new(app.tick).direction(Direction::Vertical),
//!         area,
//!     );
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod circle_spinner;
mod linear_spinner;
mod rect_spinner;
mod square_spinner;

pub use circle_spinner::CircleSpinner;
pub use linear_spinner::{Direction, LinearSpinner, LinearStyle};
pub use rect_spinner::{Centre, RectShape, RectSpinner, RectStyle, Spin};
pub use square_spinner::{SquareSpinner, SquareStyle};
