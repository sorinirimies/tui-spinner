#![no_main]
mod common;
use arbitrary::Arbitrary;
use common::*;
use libfuzzer_sys::fuzz_target;
use tui_spinner::*;

#[derive(Arbitrary, Debug)]
struct Input {
    tick: Tick,
    width: Option<Num>,
    height: Option<Num>,
    ticks_per_step: Option<Tick>,
    frames: Option<u8>,
    frame: Frame,
    size: Size,
}

fuzz_target!(|i: Input| {
    let mut s = FluxSpinner::new(i.tick.get());
    if let Some(n) = i.width { s = s.width(n.get()); }
    if let Some(n) = i.height { s = s.height(n.get()); }
    if let Some(n) = i.ticks_per_step { s = s.ticks_per_step(n.get()); }
    if let Some(k) = i.frames {
        s = s.frames(match k % 5 {
            0 => FluxFrames::BRAILLE,
            1 => FluxFrames::ORBIT,
            2 => FluxFrames::CLASSIC,
            3 => FluxFrames::LINE,
            _ => &[],
        });
    }
    if let Some(b) = i.frame.block() { s = s.block(b); }
    s = s.style(i.frame.style());
    let _ = s.char_size();
    render(&s, i.size);
});
