#![cfg(feature = "theme")]
//! Integration tests for the shared `tui-nuances` theming API.

use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};
use tui_spinner::{
    BarSpinner, CircleSpinner, FluxSpinner, LinearSpinner, RectSpinner, SquareSpinner, Theme,
    Themed,
};

fn render<W: Widget>(w: W) -> Buffer {
    let area = Rect::new(0, 0, 30, 12);
    let mut buf = Buffer::empty(area);
    w.render(area, &mut buf);
    buf
}

#[test]
fn no_theme_is_unchanged() {
    assert_eq!(
        render(BarSpinner::new(3)),
        render(BarSpinner::new(3).style(ratatui::style::Style::default()))
    );
}

#[test]
fn theme_maps_roles() {
    let t = Theme::nord();
    let b = BarSpinner::new(0).theme(&t);
    let expect = BarSpinner::new(0)
        .with_colors(t.accent, t.disabled)
        .style(t.style());
    assert_eq!(render(b), render(expect));
    let f = FluxSpinner::new(0).theme(&t);
    assert_eq!(
        render(f),
        render(FluxSpinner::new(0).color(t.accent).style(t.style()))
    );
}

#[test]
fn explicit_colors_win_in_any_order() {
    let t = Theme::dracula();
    let before = CircleSpinner::new(1).arc_color(Color::Red).theme(&t);
    let after = CircleSpinner::new(1).theme(&t).arc_color(Color::Red);
    let want = CircleSpinner::new(1)
        .arc_color(Color::Red)
        .dim_color(t.disabled)
        .style(t.style());
    assert_eq!(render(before), render(want.clone()));
    assert_eq!(render(after), render(want));
}

#[test]
fn explicit_style_wins() {
    let t = Theme::nord();
    let st = ratatui::style::Style::default().bg(Color::Red);
    let a = LinearSpinner::new(0).style(st).theme(&t);
    let b = LinearSpinner::new(0).theme(&t).style(st);
    assert_eq!(render(a), render(b));
}

#[test]
fn every_theme_renders_every_widget() {
    for t in Theme::all() {
        render(BarSpinner::new(2).theme(&t));
        render(CircleSpinner::new(2).theme(&t));
        render(FluxSpinner::new(2).theme(&t));
        render(LinearSpinner::new(2).theme(&t));
        render(RectSpinner::new(2).theme(&t));
        render(SquareSpinner::new(2).theme(&t));
        render(SquareSpinner::new(2).apply_theme(&t));
    }
}

#[test]
fn provider_works() {
    let _ = FluxSpinner::new(0).themed(&Theme::mono());
}
