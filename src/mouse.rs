
use crate::graphics::{
    WRITER,
    PixelColor,
};
use crate::sync::SpinMutex;

pub const MOUSE_CURSOR_WIDTH: usize = 15;
pub const MOUSE_CURSOR_HEIGHT: usize = 24;
pub const MOUSE_CURSOR_SHAPE: [&'static str; MOUSE_CURSOR_HEIGHT] = [
    "@              ",
    "@@             ",
    "@.@            ",
    "@..@           ",
    "@...@          ",
    "@....@         ",
    "@.....@        ",
    "@......@       ",
    "@.......@      ",
    "@........@     ",
    "@.........@    ",
    "@..........@   ",
    "@...........@  ",
    "@............@ ",
    "@......@@@@@@@@",
    "@......@       ",
    "@....@@.@      ",
    "@...@ @.@      ",
    "@..@   @.@     ",
    "@.@    @.@     ",
    "@@      @.@    ",
    "@       @.@    ",
    "         @.@   ",
    "         @@@   ",
];

pub enum MouseButton {
    Left = 0b001,
    Right = 0b010,
    Middle = 0b100,
}




pub extern "C" fn mouse_observer(buttons: u8, displacement_x: i8, displacement_y: i8) {
    let mut mouse = MOUSE.lock();
    let (x, y) = (displacement_x as usize, displacement_y as usize);
    mouse.move_relative(x, y);
}

pub static MOUSE: SpinMutex<Mouse> = SpinMutex::new(Mouse::new(200, 300));
pub struct Mouse {
    x: usize,
    y: usize,
}

use core::fmt;
impl fmt::Display for SpinMutex<Mouse> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mouse = self.lock().pos();
        write!(f, "mouse({}, {})", mouse.0, mouse.1)
    }
}

impl Mouse {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    pub fn move_relative(&mut self, x: usize, y: usize) {
        self.erase();
        self.x += x;
        self.y += y;
        self.draw();
    }

    pub fn erase(&self) {
        let writer = unsafe { WRITER.lock().assume_init() };
        for dy in 0..MOUSE_CURSOR_SHAPE.len() {
            MOUSE_CURSOR_SHAPE[dy].chars()
                .enumerate()
                .for_each(|(dx, c)| {
                    match c {
                        '@' | '.' => writer.draw_pixel(self.x+dx, self.y+dy, &PixelColor::WHITE),
                        ' ' => { /* do nothing */ },
                        c => panic!("Unexpected cursor shape: {}", c),
                    }
            });
        }

    }

    pub fn draw(&self) {
        let writer = unsafe { WRITER.lock().assume_init() };
        for dy in 0..MOUSE_CURSOR_SHAPE.len() {
            MOUSE_CURSOR_SHAPE[dy].chars()
                .enumerate()
                .for_each(|(dx, c)| {
                    match c {
                        '@' => writer.draw_pixel(self.x+dx, self.y+dy, &PixelColor::BLACK),
                        '.' => writer.draw_pixel(self.x+dx, self.y+dy, &PixelColor::WHITE),
                        ' ' => { /* do nothing */ },
                        c => panic!("Unexpected cursor shape: {}", c),
                    }
            });
        }
        
    }
}
