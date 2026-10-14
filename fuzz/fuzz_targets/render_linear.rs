#![no_main]
mod common;
use arbitrary::Arbitrary;
use common::*;
use libfuzzer_sys::fuzz_target;
use tui_spinner::*;

#[derive(Arbitrary, Debug)]
struct Input {
    tick: Tick,
    total: Option<Num>,
    lit: Option<Num>,
    ticks_per_step: Option<Tick>,
    direction: Option<bool>,
    flow: Option<bool>,
    style: Option<u8>,
    frame: Frame,
    size: Size,
}

fuzz_target!(|i: Input| {
    let mut s = LinearSpinner::new(i.tick.get());
    if let Some(n) = i.total { s = s.total_slots(n.get()); }
    if let Some(n) = i.lit { s = s.lit_slots(n.get()); }
    if let Some(n) = i.ticks_per_step { s = s.ticks_per_step(n.get()); }
    if let Some(h) = i.direction {
        s = s.direction(if h { Direction::Horizontal } else { Direction::Vertical });
    }
    if let Some(f) = i.flow {
        s = s.flow(if f { Flow::Forwards } else { Flow::Backwards });
    }
    if let Some(k) = i.style {
        s = s.linear_style(match k % 6 {
            0 => LinearStyle::Classic,
            1 => LinearStyle::Square,
            2 => LinearStyle::Diamond,
            3 => LinearStyle::Bar,
            4 => LinearStyle::Braille,
            _ => LinearStyle::Arrow,
        });
    }
    if let Some(b) = i.frame.block() { s = s.block(b); }
    s = s.style(i.frame.style());
    render(&s, i.size);
});
