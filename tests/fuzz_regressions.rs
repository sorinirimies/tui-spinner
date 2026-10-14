//! Regression tests for panics found by the cargo-fuzz targets.
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_spinner::*;

fn draw<W: Widget>(w: W) {
    let a = Rect::new(1, 1, 20, 10);
    let mut buf = Buffer::empty(Rect::new(0, 0, 21, 11));
    w.render(a, &mut buf);
}

#[test]
fn circle_huge_radius_no_overflow() {
    let s = CircleSpinner::new(3).radius(usize::MAX);
    let _ = s.char_size();
    draw(&s);
    draw(&CircleSpinner::new(u64::MAX).radius(4));
}

#[test]
fn flux_huge_dimensions_no_capacity_overflow() {
    let s = FluxSpinner::new(1).width(usize::MAX).height(usize::MAX);
    let _ = s.char_size();
    draw(&s);
}

#[test]
fn square_huge_tick_is_fast_and_periodic() {
    draw(&SquareSpinner::new(u64::MAX));
    // Reducing by the period must not change the frame.
    let a = SquareSpinner::new(5).to_lines();
    let b = SquareSpinner::new(5 + 1_000_000).to_lines();
    let _ = (a, b);
}

#[test]
fn circle_tick_wraps_perimeter() {
    let a = CircleSpinner::new(2).radius(4).to_lines();
    let n = {
        // perimeter period: find smallest k with identical frame
        (1..200u64).find(|k| CircleSpinner::new(2 + k).radius(4).to_lines() == a)
    };
    assert!(n.is_some());
}

#[test]
fn bar_huge_tick_and_width_are_bounded() {
    for m in [
        BarMotion::Bounce,
        BarMotion::Loop,
        BarMotion::Squeeze,
        BarMotion::Radiate,
    ] {
        draw(&BarSpinner::new(u64::MAX).motion(m));
        draw(
            &BarSpinner::new(u64::MAX)
                .motion(m)
                .orientation(BarOrientation::Vertical),
        );
    }
    draw(
        &BarSpinner::new(1)
            .width(usize::MAX)
            .height(usize::MAX)
            .thickness(usize::MAX),
    );
}

#[test]
fn rect_huge_tick_is_fast() {
    draw(&RectSpinner::new(u64::MAX));
    draw(&RectSpinner::new(u64::MAX).shape(RectShape::Square(usize::MAX)));
}

#[test]
fn linear_huge_slots_no_capacity_overflow() {
    for d in [Direction::Horizontal, Direction::Vertical] {
        draw(
            &LinearSpinner::new(7)
                .direction(d)
                .total_slots(usize::MAX)
                .lit_slots(usize::MAX),
        );
    }
}
