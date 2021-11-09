
use uefi::proto::console::gop::PixelFormat as UEFIPixelFormat;
use core::slice;

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

use uefi::prelude::{SystemTable, Boot, ResultExt};

impl FrameBuffer {
    pub fn from_system_table(system_table: &SystemTable<Boot>) -> Self {
        use uefi::proto::console::gop::GraphicsOutput;
        let protocol = system_table.boot_services()
            .locate_protocol::<GraphicsOutput>().unwrap_success(); 
        let gop = unsafe { &mut *protocol.get() }; 
        
        // frame buffer 
        let frame_buffer_ptr = gop.frame_buffer().as_mut_ptr();

        let mode_info = gop.current_mode_info();
        // pixel per scan line
        let pixel_per_scan_line = mode_info.stride();
        // resolution
        let resolution = mode_info.resolution();
        // pixel format
        let pixel_format = match mode_info.pixel_format() {
            UEFIPixelFormat::Rgb => {
                PixelFormat::PixelRGBResv8BitPerColor
            },
            UEFIPixelFormat::Bgr => {
                PixelFormat::PixelBGRResv8BitPerColor
            },
            UEFIPixelFormat::Bitmask => panic!("unexpected PixelFormat::Bitmask"),
            UEFIPixelFormat::BltOnly => panic!("unexpected PixelFormat::BltOnly"),
        };

        Self {
            frame_buffer: frame_buffer_ptr,
            pixel_per_scan_line,
            horizontal_resolution: resolution.0,
            vertical_resolution: resolution.1,
            pixel_format,
        }
    }
}







