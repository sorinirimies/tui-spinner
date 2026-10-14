#![no_main]
mod common;
use arbitrary::Arbitrary;
use common::*;
use libfuzzer_sys::fuzz_target;
use tui_spinner::*;

#[derive(Arbitrary, Debug)]
struct Input {
    tick: Tick,
    preset: u8,
    width: Option<Num>,
    height: Option<Num>,
    arc_width: Option<Num>,
    ticks_per_step: Option<Tick>,
    fade_width: Option<Num>,
    thickness: Option<Num>,
    arc_char: Option<u8>,
    track: Option<(u8, u8)>,
    style: Option<u8>,
    motion: Option<u8>,
    orient: Option<u8>,
    colors: Option<(Col, Col)>,
    frame: Frame,
    size: Size,
    lines: (u8, u8),
}

fuzz_target!(|i: Input| {
    let t = i.tick.get();
    let mut s = match i.preset % 5 {
        0 => BarSpinner::new(t),
        1 => BarSpinner::zed(t),
        2 => BarSpinner::claude(t),
        3 => BarSpinner::minimal(t),
        _ => BarSpinner::solid(t),
    };
    if let Some(n) = i.width { s = s.width(n.get()); }
    if let Some(n) = i.height { s = s.height(n.get()); }
    if let Some(n) = i.arc_width { s = s.arc_width(n.get()); }
    if let Some(n) = i.ticks_per_step { s = s.ticks_per_step(n.get()); }
    if let Some(n) = i.fade_width { s = s.fade_width(n.get()); }
    if let Some(n) = i.thickness { s = s.thickness(n.get()); }
    if let Some(b) = i.arc_char { s = s.arc_char(b); }
    if let Some((k, b)) = i.track {
        s = s.track(match k % 4 {
            0 => BarTrack::Rail,
            1 => BarTrack::Full,
            2 => BarTrack::Empty,
            _ => BarTrack::Custom(b),
        });
    }
    if let Some(k) = i.style {
        s = s.bar_style(match k % 9 {
            0 => BarStyle::Braille,
            1 => BarStyle::Block,
            2 => BarStyle::Shade,
            3 => BarStyle::Dot,
            4 => BarStyle::Diamond,
            5 => BarStyle::Square,
            6 => BarStyle::Star,
            7 => BarStyle::Heart,
            _ => BarStyle::Arrow,
        });
    }
    if let Some(k) = i.motion {
        s = s.motion(match k % 4 {
            0 => BarMotion::Bounce,
            1 => BarMotion::Loop,
            2 => BarMotion::Squeeze,
            _ => BarMotion::Radiate,
        });
    }
    if let Some(k) = i.orient {
        s = s.orientation(if k % 2 == 0 { BarOrientation::Horizontal } else { BarOrientation::Vertical });
    }
    if let Some((a, b)) = i.colors { s = s.with_colors(a.get(), b.get()); }
    if let Some(b) = i.frame.block() { s = s.block(b); }
    s = s.style(i.frame.style());
    let _ = s.char_size();
    // Bounded line rendering to avoid OOM on huge dimensions.
    let _ = s.to_lines(i.lines.0 as usize, i.lines.1 as usize);
    render(&s, i.size);
});
