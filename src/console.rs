
use crate::graphics::{
    PixelColor, FrameBuffer, Font, ShinonomeFont
};

#[derive(Clone, Copy)]
struct Color {
    pub fg: PixelColor,
    pub bg: PixelColor,
}

impl Color {
    pub const DEFAULT: Self = Self {
        fg: PixelColor::BLACK,
        bg: PixelColor::WHITE,
    };
}

#[derive(Clone, Copy)]
struct ConsoleChar {
    ch: Option<char>,
    color: Color,
}

impl ConsoleChar {
    pub const DEFAULT: Self = Self {
        ch: None,
        color: Color::DEFAULT,
    };
}

#[derive(Clone, Copy)]
struct Cursor {
    x: usize,
    y: usize,
}

pub struct Console {
    rows: usize, // <= 600/16 (== QEMU window size / hankaku font vertical length) < 40
    columns: usize, // <= 800/8 = 100
    buffer: [char; 10000],
    color: Color,
    cursor: Cursor,
    scroll_flag: bool,
}

const ROWS: usize = 10;
const COLUMNS: usize =  80;

use crate::graphics::PixelWriter;
impl Console {
    pub const fn new() -> Self {
        Self {
            rows: ROWS,
            columns: COLUMNS,
            buffer: [' '; 10000],
            color: Color::DEFAULT,
            cursor: Cursor {x: 0, y: 0},
            scroll_flag: false,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn columns(&self) -> usize {
        self.columns
    }
    pub fn fg(&self) -> PixelColor {
        self.color.fg
    }
    pub fn bg(&self) -> PixelColor {
        self.color.bg
    }


    pub fn render(&mut self, writer: &dyn PixelWriter, font: &dyn Font) {
        let (font_x, font_y) = font.char_size();
        for y in 0..self.rows {
            for x in 0..self.columns {
                let index = y*self.columns + x;
                let ch = self.buffer[index];
                use crate::graphics::Vector2D;
                if self.scroll_flag {
                    writer.fill_rect(
                        Vector2D::new(x*font_x, y*font_y),
                        Vector2D::new(font_x, font_y),
                        &self.color.bg,
                    );
                }
                font.write_ascii(writer, font_x*x, font_y*y, ch, &self.color.fg, &self.color.bg);
            }
        }
        self.scroll_flag = false;
    }

    pub fn put_string(&mut self, s: &str) {
        s.chars().for_each(|c| {
            if c == '\n' { self.new_line() }
            else if self.cursor.x < self.columns - 1 {
                let index = self.cursor.y * self.columns + self.cursor.x;
                self.buffer[index] = c;
                self.cursor.x += 1;
            }
            // TODO: else  { todo!() }
            // (currently it stops storing s to self.buffer when self.cursor.x > self.columns)
        });
    }

    fn move_cursor_forward(&mut self) {
        self.cursor.x += 1;
        if self.cursor.x == self.columns {
            self.cursor.x = 0;
            if self.cursor.y + 1 == self.rows {
                self.scroll_up();
            } else {
                self.cursor.y += 1;
            }
        }
    }

    fn move_cursor_backward(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        }
    }

    fn new_line(&mut self) {
        self.cursor.x = 0;
        if self.cursor.y == self.rows - 1 {
            // スクロールの必要あり
            self.scroll_up();
        } else {
            // スクロールの必要なし
            self.cursor.y += 1;
        }
    }

    fn scroll_up(&mut self) {
        self.scroll_flag = true;
        let end = self.rows * self.columns;
        let src = self.columns..end;
        self.buffer.copy_within(src, 0);
        self.buffer[(end-self.columns)..end].fill(' ');
    }

}

use core::fmt;
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.put_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::console::_kprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => ($crate::kprint!("{}\n", format_args!($($arg)*)));
}


// need static CONSOLE for kprint! macro.
// TODO: 1. implement spin mutex -> FINISHED
// TODO: 2. initialize WRITER (frame buffer) in kernel_main -> FINISHED
// TODO: 3. really need spin mutex?
// TODO: 4. is the implementation correct?
pub static CONSOLE: SpinMutex<Console> = SpinMutex::new(
    Console::new()
);

pub static CONSOLE_FONT: ShinonomeFont = ShinonomeFont::new();


use crate::graphics::WRITER;
pub fn _kprint(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut console = CONSOLE.lock();
    let writer = WRITER.lock();
    let writer = unsafe { writer.assume_init() };
    console.write_fmt(args).unwrap();
    console.render(writer, &CONSOLE_FONT);
}

#[no_mangle]
pub extern "C" fn usb_log(_level: i32, msg: *const u8, msg_len: i32) {
    let s = unsafe { core::slice::from_raw_parts(msg, msg_len as usize) };
    let s = unsafe { core::str::from_utf8_unchecked(s) };
    // kprint!("{}", s);
}

// ------------------------------------------------------
// SpinMutex
// ------------------------------------------------------
// TODO: move spinmutex to spinmutex_like.rs
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
pub struct SpinMutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>
}

impl<T> SpinMutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn try_lock(&self) -> Result<SpinMutexGuard<T>, SpinMutexErr> {
        if !self.lock.swap(true, Ordering::Acquire) {
            Ok(SpinMutexGuard { mutex: self })
        } else {
            Err(SpinMutexErr("lock error"))
        }

    }

    pub fn lock(&self) -> SpinMutexGuard<T> {
        loop {
            if let Ok(guard) = self.try_lock() {
                return guard;
            }
        }
    }

}

// Send + Sync are required for static 
unsafe impl<T> Send for SpinMutex<T> {}
unsafe impl<T> Sync for SpinMutex<T> {}

pub struct SpinMutexGuard<'a, T> {
    mutex: &'a SpinMutex<T>,
}

impl<T> SpinMutexGuard<'_, T> {
    fn unlock(&self) {
        self.mutex.lock.swap(false, Ordering::Release);
    }
}

// when drop, unlock
use core::ops::Drop;
impl<T> Drop for SpinMutexGuard<'_, T> {
    fn drop(&mut self) {
        self.unlock();
    }
}

use core::ops::Deref;
impl<T> Deref for SpinMutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

use core::ops::DerefMut;
impl<T> DerefMut for SpinMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

pub struct SpinMutexErr<'a>(&'a str);



