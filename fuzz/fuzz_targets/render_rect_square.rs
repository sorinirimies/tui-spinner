#![no_main]
mod common;
use arbitrary::Arbitrary;
use common::*;
use libfuzzer_sys::fuzz_target;
use tui_spinner::*;

#[derive(Arbitrary, Debug)]
enum Input {
    Rect {
        tick: Tick,
        shape: Num,
        ccw: bool,
        filled: bool,
        ticks_per_step: Option<Tick>,
        frame: Frame,
        size: Size,
    },
    Square {
        tick: Tick,
        size_n: Option<Num>,
        ticks_per_step: Option<Tick>,
        frame: Frame,
        size: Size,
    },
}

fuzz_target!(|i: Input| match i {
    Input::Rect { tick, shape, ccw, filled, ticks_per_step, frame, size } => {
        let mut s = RectSpinner::new(tick.get());
        s = s.shape(RectShape::Square(shape.get()));
        s = s.spin(if ccw { Spin::CounterClockwise } else { Spin::Clockwise });
        s = s.centre(if filled { Centre::Filled } else { Centre::Empty });
        if let Some(n) = ticks_per_step { s = s.ticks_per_step(n.get()); }
        if let Some(b) = frame.block() { s = s.block(b); }
        s = s.style(frame.style());
        render(&s, size);
    }
    Input::Square { tick, size_n, ticks_per_step, frame, size } => {
        let mut s = SquareSpinner::new(tick.get());
        if let Some(n) = size_n { s = s.size(n.get()); }
        if let Some(n) = ticks_per_step { s = s.ticks_per_step(n.get()); }
        if let Some(b) = frame.block() { s = s.block(b); }
        s = s.style(frame.style());
        let _ = s.char_size();
        render(&s, size);
    }
});
