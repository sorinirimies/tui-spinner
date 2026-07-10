// Shared internal macros for spinner widget boilerplate.
// Loaded via `#[macro_use] mod macros;` in lib.rs — available in every spinner module.

// ── impl Styled ───────────────────────────────────────────────────────────────

/// Implements `ratatui::style::Styled` for a spinner type.
///
/// Requires the type to have a `style: Style` field.
///
/// ```text
/// impl_styled_for!(BarSpinner<'_>);
/// ```
macro_rules! impl_styled_for {
    ($t:ty) => {
        impl ratatui::style::Styled for $t {
            type Item = Self;

            fn style(&self) -> ratatui::style::Style {
                self.style
            }

            fn set_style<S: Into<ratatui::style::Style>>(mut self, style: S) -> Self {
                self.style = style.into();
                self
            }
        }
    };
}

// ── impl Widget (owned → ref delegation) ─────────────────────────────────────

/// Implements `Widget for T` by delegating to `Widget for &T`.
///
/// Every spinner provides a `impl Widget for &T` that does the real work;
/// this generates the boilerplate `impl Widget for T` wrapper.
///
/// ```text
/// impl_widget_via_ref!(BarSpinner<'_>);
/// ```
macro_rules! impl_widget_via_ref {
    ($t:ty) => {
        impl ratatui::widgets::Widget for $t {
            fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
                ratatui::widgets::Widget::render(&self, area, buf);
            }
        }
    };
}

// ── Standard widget render body ───────────────────────────────────────────────

/// Renders the standard spinner body inside `impl Widget for &T`.
///
/// 1. Early-exit on zero area.
/// 2. Apply the base style to the full area.
/// 3. Resolve the optional `Block` wrapper.
/// 4. Early-exit if the inner area is also zero.
/// 5. Render `$lines` via `Paragraph` with the widget's alignment.
///
/// ```text
/// impl Widget for &BarSpinner<'_> {
///     fn render(self, area: Rect, buf: &mut Buffer) {
///         render_spinner_body!(self, area, buf, self.build_lines(width));
///     }
/// }
/// ```
macro_rules! render_spinner_body {
    ($self:ident, $area:ident, $buf:ident, $lines:expr) => {{
        if $area.area() == 0 {
            return;
        }

        $buf.set_style($area, $self.style);

        let inner_area = $self.block.as_ref().map_or($area, |b| {
            let inner = b.inner($area);
            ratatui::widgets::Widget::render(b.clone(), $area, $buf);
            inner
        });

        if inner_area.area() == 0 {
            return;
        }

        ratatui::widgets::Paragraph::new($lines)
            .alignment($self.alignment)
            .render(inner_area, $buf);
    }};
}

// ── Embeddable text conversion ────────────────────────────────────────────────

/// Implements `to_lines()` / `to_text()` **and** `From<Spinner> for Text` for a
/// spinner type, so its current frame can be embedded in any widget that
/// accepts `Into<Text>` content.
///
/// Because `ratatui`'s `Cell` and `Paragraph::new` accept `Into<Text>`, the
/// generated `From` impls make the following all work with no special method:
///
/// ```text
/// Cell::from(&spinner)        // by reference
/// Cell::from(spinner)         // by value
/// Paragraph::new(&spinner)
/// let text: Text = spinner.into();
/// ```
///
/// `to_lines()` remains available for the case where the spinner rows need to
/// be combined with other lines in the same cell.
///
/// `$builder` is the private method that produces the `Vec<Line>` for the
/// current frame (typically `build_lines` or `render_lines`).
///
/// ```text
/// impl_to_text!(FluxSpinner<'_>, build_lines);
/// ```
macro_rules! impl_to_text {
    ($t:ty, $builder:ident) => {
        impl $t {
            /// Renders the current frame as a `Vec<Line>`, one
            /// [`Line`](ratatui::text::Line) per row.
            ///
            /// Use this when the spinner rows need to be combined with other
            /// lines in the same cell. To embed the whole spinner on its own,
            /// prefer the `Into<Text>` conversion (e.g. `Cell::from(&spinner)`).
            #[must_use]
            pub fn to_lines(&self) -> Vec<ratatui::text::Line<'static>> {
                self.$builder()
            }

            /// Renders the current frame as a
            /// [`Text`](ratatui::text::Text) value.
            ///
            /// Equivalent to `Text::from(&spinner)`; kept for discoverability.
            #[must_use]
            pub fn to_text(&self) -> ratatui::text::Text<'static> {
                ratatui::text::Text::from(self.$builder())
            }
        }

        impl From<&$t> for ratatui::text::Text<'static> {
            fn from(spinner: &$t) -> Self {
                ratatui::text::Text::from(spinner.$builder())
            }
        }

        impl From<$t> for ratatui::text::Text<'static> {
            fn from(spinner: $t) -> Self {
                ratatui::text::Text::from(spinner.$builder())
            }
        }
    };
}
