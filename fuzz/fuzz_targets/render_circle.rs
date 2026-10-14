#![no_main]
mod common;
use arbitrary::Arbitrary;
use common::*;
use libfuzzer_sys::fuzz_target;
use tui_spinner::*;

#[derive(Arbitrary, Debug)]
struct Input {
    tick: Tick,
    radius: Option<Num>,
    arc_len: Option<Num>,
    ticks_per_step: Option<Tick>,
    frame: Frame,
    size: Size,
}

fuzz_target!(|i: Input| {
    let mut s = CircleSpinner::new(i.tick.get());
    if let Some(n) = i.radius { s = s.radius(n.get()); }
    if let Some(n) = i.arc_len { s = s.arc_len(n.get()); }
    if let Some(n) = i.ticks_per_step { s = s.ticks_per_step(n.get()); }
    if let Some(b) = i.frame.block() { s = s.block(b); }
    s = s.style(i.frame.style());
    let _ = s.char_size();
    render(&s, i.size);
});
