
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




pub extern "C" fn mouse_observer(dx: i8, dy: i8) {
    // kprintln!("mouse_observer({}, {})", dx, dy);
    let mut mouse = MOUSE.lock();
    let (dx, dy) = (dx as isize, dy as isize);
    mouse.move_relative(dx, dy);
}

pub static MOUSE: SpinMutex<Mouse> = SpinMutex::new(Mouse::new());
pub fn init_mouse() {
    let mut mouse = MOUSE.lock();
    mouse.init(200, 300);
    mouse.draw();
}

pub struct Mouse {
    x: isize,
    y: isize,
    max_x: isize,
    max_y: isize,
}

use core::fmt;
impl fmt::Display for SpinMutex<Mouse> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mouse = self.lock().pos();
        write!(f, "mouse({}, {})", mouse.0, mouse.1)
    }
}

impl Mouse {
    pub const fn new() -> Self {
        Self { x: 0, y: 0, max_x: 0, max_y: 0 }
    }

    pub fn init(&mut self, x: isize, y: isize) {
        (self.x, self.y) = (x, y);
        let writer = unsafe { WRITER.lock().assume_init() };
        (self.max_x, self.max_y) = (writer.horizontal_resolution() as isize, writer.vertical_resolution() as isize);
    }

    pub fn pos(&self) -> (isize, isize) {
        (self.x, self.y)
    }

    pub fn move_relative(&mut self, dx: isize, dy: isize) {
        // 1. if x + self.x < 0 { self.x = 0 }
        // 2. else if x + self.x > self.max_x { self.x = self.max_x }
        // 3. else { self.x += x }
        self.erase();
        // todo: usize でもつなら self.x + dx で負になるか事前に判定
        self.x = match self.x + dx {
            v if v < 0 => { 0 },
            v if v > self.max_x => { self.max_x },
            v => { v },
        };
        self.y = match self.y + dy {
            v if v < 0 => { 0 },
            v if v > self.max_y => { self.max_y },
            v => { v },
        };
        self.draw();
    }

    // todo: 色を指定できるようにする
    pub fn erase(&self) {
        let writer = unsafe { WRITER.lock().assume_init() };
        for dy  in 0..MOUSE_CURSOR_SHAPE.len() {
            MOUSE_CURSOR_SHAPE[dy].chars()
                .enumerate()
                .for_each(|(dx, c)| {
                    let x = self.x as usize + dx;
                    let y = self.y as usize + dy;
                    match c {
                        '@' | '.' => writer.draw_pixel(x, y, &PixelColor::WHITE),
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
                    let x = self.x as usize + dx;
                    let y = self.y as usize + dy;
                    match c {
                        '@' => writer.draw_pixel(x, y, &PixelColor::BLACK),
                        '.' => writer.draw_pixel(x, y, &PixelColor::WHITE),
                        ' ' => { /* do nothing */ },
                        c => panic!("Unexpected cursor shape: {}", c),
                    }
            });
        }
        
    }
}
