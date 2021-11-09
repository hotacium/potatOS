
pub struct PixelColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl PixelColor {
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
    pub fn draw_pixel(&self, x: usize, y: usize, color: &PixelColor) {
        let pixel_position = self.pixel_per_scan_line * y + x;
        match self.pixel_format {
            PixelFormat::PixelRGBResv8BitPerColor => {
                let pixel = unsafe { self.frame_buffer.add(4*pixel_position) };
                let color_data = [color.red, color.green, color.blue];
                for (i, &item) in color_data.iter().enumerate() {
                    unsafe { pixel.add(i).write_volatile(item) };
                }
            },
            PixelFormat::PixelBGRResv8BitPerColor => {
                let pixel = unsafe { self.frame_buffer.add(4*pixel_position) };
                let color_data = [color.blue, color.green, color.red];
                for (i, &item) in color_data.iter().enumerate() {
                    unsafe { pixel.add(i).write_volatile(item) };
                }
            },
        }
    }

    pub fn h(&self) -> usize {
        self.horizontal_resolution
    }

    pub fn v(&self) -> usize {
        self.vertical_resolution
    }

}
