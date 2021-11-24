
#[derive(Clone, Copy)]
pub struct PixelColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl PixelColor {
    pub const BLACK: Self = Self {
        red: 0, green: 0, blue: 0,
    };
    pub const WHITE: Self = Self {
        red: 255, green: 255, blue: 255,
    };
    pub const RED: Self = Self {
        red: 255, green: 0, blue: 0,
    };
    pub const GREEN: Self = Self {
        red: 0, green: 255, blue: 0,
    };
    pub const BLUE: Self = Self {
        red: 0, green: 0, blue: 255,
    };
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            red: r,
            green: g,
            blue: b,
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum PixelFormat {
    PixelRGBResv8BitPerColor,
    PixelBGRResv8BitPerColor,
}


// need init CONSOLE_WRITER in kernel_main
use crate::console::SpinMutex;
use core::mem::MaybeUninit;
pub static WRITER: SpinMutex<MaybeUninit<&dyn PixelWriter>> = SpinMutex::new(
    MaybeUninit::<&dyn PixelWriter>::uninit()
);


#[derive(Debug)]
#[repr(C)]
pub struct FrameBuffer {
    frame_buffer: *mut u8,
    pixel_per_scan_line: usize,
    horizontal_resolution: usize,
    vertical_resolution: usize,
    pixel_format: PixelFormat,
}

impl FrameBuffer {
    pub const fn uninitialized_default() -> Self {
        Self {
            frame_buffer: 0 as *mut u8,
            pixel_per_scan_line: 0,
            horizontal_resolution: 0,
            vertical_resolution: 0,
            pixel_format: PixelFormat::PixelRGBResv8BitPerColor,
        }
    }

    pub fn h(&self) -> usize {
        self.horizontal_resolution
    }

    pub fn v(&self) -> usize {
        self.vertical_resolution
    }

    pub fn pixel_format(&self) -> &PixelFormat {
        &self.pixel_format
    }

}

impl PixelWriter for FrameBuffer {
    fn draw_pixel(&self, x: usize, y: usize, color: &PixelColor) {
        let pixel_position = self.pixel_per_scan_line * y + x;
        let color_data = match self.pixel_format {
            PixelFormat::PixelRGBResv8BitPerColor => [color.red, color.green, color.blue],
            PixelFormat::PixelBGRResv8BitPerColor => [color.blue, color.green, color.red],
        };
        let pixel = unsafe { self.frame_buffer.add(4*pixel_position) };
        for (i, &item) in color_data.iter().enumerate() {
            unsafe { pixel.add(i).write_volatile(item) };
        }
    }
}

pub trait PixelWriter {
    // fn new(frame_buffer: FrameBuffer) -> Self;
    // fn init(&mut self, fb: Self) {
    //     let _old_self = core::mem::replace(self, fb);
    // }

    fn draw_pixel(&self, x:usize, y:usize, color: &PixelColor);

    fn fill_rect(&self, pos: Vector2D<usize>, size: Vector2D<usize>, color: &PixelColor) {
        for dy in 0..size.y() {
            for dx in 0..size.x() {
                self.draw_pixel(pos.x() + dx, pos.y() + dy, color)
            }
        }
    }

    fn draw_rect(&self, pos: Vector2D<usize>, size: Vector2D<usize>, color: &PixelColor) {
        for dx in 0..size.x() {
            self.draw_pixel(pos.x()+dx, pos.y(), color);
            self.draw_pixel(pos.x()+dx, pos.y()+size.y(), color);
        }
        for dy in 0..size.y() {
            self.draw_pixel(pos.x(), pos.y()+dy, color);
            self.draw_pixel(pos.x()+size.x(), pos.y()+dy, color);
        }
    }
}

pub struct RGBResv8BitPerColorPixelWriter {
    frame_buffer: FrameBuffer
}

impl RGBResv8BitPerColorPixelWriter {
    pub fn new(frame_buffer: FrameBuffer) -> Self {
        Self { frame_buffer }
    }
}

impl PixelWriter for RGBResv8BitPerColorPixelWriter {
    fn draw_pixel(&self, x:usize, y:usize, color: &PixelColor) {
        let pixel_position = self.frame_buffer.pixel_per_scan_line * y + x;
        let pixel = unsafe { self.frame_buffer.frame_buffer.add(4*pixel_position) };
        let color_data = [color.red, color.green, color.blue];
        for (i, &item) in color_data.iter().enumerate() {
            unsafe { pixel.add(i).write_volatile(item) };
        }
    }
}

pub struct BGRResv8BitPerColorPixelWriter {
    frame_buffer: FrameBuffer
}

impl BGRResv8BitPerColorPixelWriter {
    pub fn new(frame_buffer: FrameBuffer) -> Self {
        Self { frame_buffer }
    }

}

impl PixelWriter for BGRResv8BitPerColorPixelWriter {
    fn draw_pixel(&self, x:usize, y:usize, color: &PixelColor) {
        let pixel_position = self.frame_buffer.pixel_per_scan_line * y + x;
        let pixel = unsafe { self.frame_buffer.frame_buffer.add(4*pixel_position) };
        let color_data = [color.blue, color.green, color.red];
        for (i, &item) in color_data.iter().enumerate() {
            unsafe { pixel.add(i).write_volatile(item) };
        }
    }
}

pub trait Font {
    fn char_size(&self) -> (usize, usize);
    fn write_ascii(&self, writer: &dyn PixelWriter, x: usize, y: usize, c: char, color: &PixelColor); 
    fn write_string(&self, writer: &dyn PixelWriter, x: usize, y: usize, s: &str, color: &PixelColor) {
        for (i, c) in s.chars().enumerate() {
            self.write_ascii(writer, i*8 + x, y, c, color);
        }
    }
}

pub struct ShinonomeFont {
    font: &'static [u8]
}

impl Font for ShinonomeFont {
    fn char_size(&self) -> (usize, usize) {
        (8+2, 16+2)
    }
    fn write_ascii(&self, writer: &dyn PixelWriter, x: usize, y: usize, c: char, color: &PixelColor) {
        let index = 16*c as usize;
        // if c is not ascii char
        if index >= self.font.len() {
            return
        }
        for dy in 0..16 {
            for dx in 0..8 {
                if ((self.font[index+dy] << dx) & 0x80) > 0 {
                    writer.draw_pixel(x+dx, y+dy, color);
                }
            }
        }

    }
}

impl ShinonomeFont {

    pub const fn new() -> Self {
        Self {
            font: include_bytes!("../assets/hankaku.bin"),
        }
    }

    pub fn write_string(&self, writer: &FrameBuffer, x: usize, y: usize, s: &str, color: &PixelColor) {
        for (i, c) in s.chars().enumerate() {
            self.write_ascii(writer, i*8 + x, y, c, color);
        }
    }
}


pub struct Vector2D<T: Ord + Copy> {
    x: T, 
    y: T,
}

impl<T: Ord + Copy> Vector2D<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }
}







