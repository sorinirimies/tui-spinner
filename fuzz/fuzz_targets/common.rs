//! Shared helpers: bounded arbitrary sizes and rendering into a Buffer.
#![allow(dead_code)]

use arbitrary::Arbitrary;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

pub const MAX_W: u16 = 200;
pub const MAX_H: u16 = 100;

#[derive(Arbitrary, Debug, Clone, Copy)]
pub struct Size {
    pub w: u8,
    pub h: u8,
    pub x: u8,
    pub y: u8,
}

impl Size {
    /// Area clamped to <= 200x100, offset from the origin by up to 7 cells.
    pub fn area(&self) -> Rect {
        Rect::new(
            self.x as u16 % 8,
            self.y as u16 % 8,
            (self.w as u16) % (MAX_W + 1),
            (self.h as u16) % (MAX_H + 1),
        )
    }
}

/// Render any widget into a buffer covering `size` (plus the offset).
pub fn render<W: Widget>(w: W, size: Size) {
    let a = size.area();
    let full = Rect::new(0, 0, a.x + a.width, a.y + a.height);
    let mut buf = Buffer::empty(full);
    w.render(a, &mut buf);
}

/// Pick one of a few sizes: tiny/zero values plus huge ones.
#[derive(Arbitrary, Debug, Clone, Copy)]
pub enum Num {
    Small(u8),
    Max,
    NearMax(u8),
    Any(usize),
}

impl Num {
    pub fn get(self) -> usize {
        match self {
            Num::Small(n) => n as usize,
            Num::Max => usize::MAX,
            Num::NearMax(n) => usize::MAX - n as usize,
            Num::Any(n) => n,
        }
    }
}

#[derive(Arbitrary, Debug, Clone, Copy)]
pub enum Tick {
    Small(u16),
    Max,
    Any(u64),
}

impl Tick {
    pub fn get(self) -> u64 {
        match self {
            Tick::Small(n) => n as u64,
            Tick::Max => u64::MAX,
            Tick::Any(n) => n,
        }
    }
}

#[derive(Arbitrary, Debug, Clone, Copy)]
pub struct Col(pub u8, pub u8, pub u8, pub u8);

impl Col {
    pub fn get(self) -> Color {
        match self.0 % 3 {
            0 => Color::Rgb(self.1, self.2, self.3),
            1 => Color::Indexed(self.1),
            _ => Color::Reset,
        }
    }
}

#[derive(Arbitrary, Debug, Clone, Copy)]
pub struct Frame {
    pub bordered: bool,
    pub title: Option<u8>,
    pub fg: Option<Col>,
    pub bg: Option<Col>,
}

impl Frame {
    pub fn block(&self) -> Option<Block<'static>> {
        self.bordered.then(|| {
            let b = Block::default().borders(Borders::ALL);
            match self.title {
                Some(n) => b.title("t".repeat(n as usize)),
                None => b,
            }
        })
    }
    pub fn style(&self) -> Style {
        let mut s = Style::default();
        if let Some(c) = self.fg {
            s = s.fg(c.get());
        }
        if let Some(c) = self.bg {
            s = s.bg(c.get());
        }
        s
    }
}
