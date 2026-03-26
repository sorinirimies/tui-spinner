//! Legacy [`SquareSpinner`] — thin shim over [`crate::RectSpinner`].
//!
//! All new code should use [`RectSpinner`] directly.  This module exists only
//! to keep the original public API compiling without changes.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Styled};
use ratatui::widgets::{Block, Widget};

use crate::rect_spinner::{Centre, RectShape, RectSpinner, Spin};

/// A simple square braille-arc spinner — legacy alias for [`RectSpinner`].
///
/// Renders a rotating arc around a square perimeter in braille characters.
/// Supports filled or empty center modes and clockwise/counter-clockwise directions.
///
/// # Examples
///
/// ```no_run
/// use ratatui::style::Color;
/// use tui_spinner::{Centre, SquareSpinner, Spin};
///
/// let spinner = SquareSpinner::new(42)
///     .size(2)
///     .centre(Centre::Filled)
///     .spin(Spin::Clockwise)
///     .outer_color(Color::Cyan)
///     .inner_color(Color::DarkGray);
/// ```
#[derive(Debug, Clone)]
pub struct SquareSpinner<'a> {
    inner: RectSpinner<'a>,
}

impl<'a> SquareSpinner<'a> {
    /// Creates a new [`SquareSpinner`] at the given tick.
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::SquareSpinner;
    ///
    /// let spinner = SquareSpinner::new(42);
    /// ```
    #[must_use]
    pub fn new(tick: u64) -> Self {
        Self {
            inner: RectSpinner::new(tick).shape(RectShape::Square(2)),
        }
    }

    /// Sets the arc thickness / size parameter (default: 2, range: 2–8).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::SquareSpinner;
    ///
    /// let large = SquareSpinner::new(0).size(4);
    /// ```
    #[must_use]
    pub fn size(mut self, size: usize) -> Self {
        let size = size.clamp(2, 8);
        self.inner = self.inner.shape(RectShape::Square(size));
        self
    }

    /// Sets the colour of the rotating arc.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::SquareSpinner;
    ///
    /// let spinner = SquareSpinner::new(0).outer_color(Color::Cyan);
    /// ```
    #[must_use]
    pub fn outer_color(mut self, color: Color) -> Self {
        self.inner = self.inner.outer_color(color);
        self
    }

    /// Sets the colour of the solid centre square.
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::style::Color;
    /// use tui_spinner::SquareSpinner;
    ///
    /// let spinner = SquareSpinner::new(0).inner_color(Color::Blue);
    /// ```
    #[must_use]
    pub fn inner_color(mut self, color: Color) -> Self {
        self.inner = self.inner.inner_color(color);
        self
    }

    /// Controls whether the centre is filled or empty (default: `Filled`).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{Centre, SquareSpinner};
    ///
    /// let hollow = SquareSpinner::new(0).centre(Centre::Empty);
    /// ```
    #[must_use]
    pub fn centre(mut self, centre: Centre) -> Self {
        self.inner = self.inner.centre(centre);
        self
    }

    /// Sets the rotation direction (default: `Clockwise`).
    ///
    /// # Examples
    ///
    /// ```
    /// use tui_spinner::{SquareSpinner, Spin};
    ///
    /// let ccw = SquareSpinner::new(0).spin(Spin::CounterClockwise);
    /// ```
    #[must_use]
    pub fn spin(mut self, spin: Spin) -> Self {
        self.inner = self.inner.spin(spin);
        self
    }

    /// Sets how many ticks the arc holds each position before advancing
    /// (default: 1, higher = slower).
    #[must_use]
    pub fn ticks_per_step(mut self, n: u64) -> Self {
        self.inner = self.inner.ticks_per_step(n);
        self
    }

    /// Wraps the spinner in a [`Block`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ratatui::widgets::Block;
    /// use tui_spinner::SquareSpinner;
    ///
    /// let spinner = SquareSpinner::new(0).block(Block::bordered().title("Loading…"));
    /// ```
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.inner = self.inner.block(block);
        self
    }

    /// Sets the base style applied to the widget area.
    #[must_use]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.inner = self.inner.style(style);
        self
    }
}

impl Styled for SquareSpinner<'_> {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.inner)
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.inner = self.inner.set_style(style);
        self
    }
}

impl Widget for SquareSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(self.inner, area, buf);
    }
}

impl Widget for &SquareSpinner<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self.inner, area, buf);
    }
}
