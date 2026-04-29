//! # tui-spinner
//!
//! Customizable animated spinner widgets for [Ratatui](https://github.com/ratatui/ratatui) TUI applications.
//!
//! ## Widgets
//!
//! | Widget | Motion | Key options |
//! |--------|--------|-------------|
//! | [`LinearSpinner`] | Scrolling window or bouncing dot along a straight axis | `direction`, `flow`, `linear_style` |
//! | [`SquareSpinner`] | Braille-dot arc rotating around a square ring | `size`, `spin`, `centre` |
//! | [`CircleSpinner`] | Braille-dot arc rotating around a circular ring | `radius`, `spin`, `arc_len` |
//! | [`RectSpinner`] | Braille-dot arc rotating around a configurable rectangle | `shape`, `spin`, `centre` |
//! | [`BarSpinner`] | Solid bar with a bouncing or looping glow arc | `bar_style`, `motion`, `spin`, `track`, `fade_width` |
//! | [`FluxSpinner`] | Single-character glyph cycling through a frame sequence | `frames`, `spin`, `phase_step` |
//! | [`SquareSpinner`] | Legacy alias for `RectSpinner::Square` | — |
//!
//! ## Quick start
//!
//! ```no_run
//! use ratatui::style::Color;
//! use tui_spinner::{
//!     BarMotion, BarSpinner, BarStyle, Centre, CircleSpinner,
//!     Direction, FluxFrames, FluxSpinner, Flow, LinearSpinner,
//!     LinearStyle, RectShape, RectSpinner, Spin, SquareSpinner,
//! };
//!
//! // Vertical bouncing dot
//! let v = LinearSpinner::new(42)
//!     .direction(Direction::Vertical)
//!     .linear_style(LinearStyle::Braille)
//!     .active_color(Color::Cyan);
//!
//! // Square arc, clockwise, filled centre
//! let sq = SquareSpinner::new(42)
//!     .size(3)
//!     .spin(Spin::Clockwise)
//!     .centre(Centre::Filled)
//!     .arc_color(Color::Cyan);
//!
//! // Circular arc, counter-clockwise
//! let circle = CircleSpinner::new(42)
//!     .radius(5)
//!     .spin(Spin::CounterClockwise)
//!     .arc_color(Color::Magenta);
//!
//! // Bouncing braille bar (Bounce) — fills available width automatically
//! let bounce = BarSpinner::new(42)
//!     .arc_color(Color::Cyan)
//!     .dim_color(Color::DarkGray);
//!
//! // Continuous sweep (Loop) with a Star symbol style
//! let sweep = BarSpinner::new(42)
//!     .bar_style(BarStyle::Star)
//!     .motion(BarMotion::Loop)
//!     .spin(Spin::Clockwise)
//!     .arc_color(Color::Yellow);
//!
//! // Minimal 1×1 status-bar spinner
//! let flux = FluxSpinner::new(42).color(Color::Cyan);
//!
//! // 8-wide travelling wave
//! let wave = FluxSpinner::new(42)
//!     .frames(FluxFrames::ORBIT)
//!     .width(8)
//!     .phase_step(1)
//!     .color(Color::LightBlue);
//! ```
//!
//! ## Integration
//!
//! All widgets are **stateless** — pass a monotonically-increasing `tick: u64`
//! counter (incremented once per render frame).  No mutable widget state needed.
//!
//! ```no_run
//! use ratatui::Frame;
//! use ratatui::layout::Rect;
//! use tui_spinner::{BarSpinner, BarMotion, Spin};
//!
//! struct App { tick: u64 }
//!
//! fn draw(frame: &mut Frame, area: Rect, app: &App) {
//!     frame.render_widget(
//!         BarSpinner::new(app.tick)
//!             .motion(BarMotion::Loop)
//!             .spin(Spin::Clockwise),
//!         area,
//!     );
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod bar_spinner;
mod circle_spinner;
mod flux_spinner;
mod linear_spinner;
mod rect_spinner;
mod square_spinner;

pub use bar_spinner::{BarMotion, BarSpinner, BarStyle, BarTrack};
pub use circle_spinner::CircleSpinner;
pub use flux_spinner::{FluxFrames, FluxSpinner};
pub use linear_spinner::{Direction, Flow, LinearSpinner, LinearStyle};
pub use rect_spinner::{Centre, RectShape, RectSpinner, Spin};
pub use square_spinner::SquareSpinner;
// Note: `Centre` and `Spin` are re-exported from rect_spinner.
// `SquareSpinner` uses the same `Centre` and `Spin` enums via re-export.
