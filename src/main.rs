#![no_std]
#![no_main]

use core::panic::PanicInfo;
use potatOS::frame_buffer::{FrameBuffer, PixelColor};

#[no_mangle]
pub extern "C" fn kernel_main(frame_buffer: &mut FrameBuffer) -> ! {
    // println!("Hello, world!");
    for x in 0..frame_buffer.h() {
        for y in 0..frame_buffer.v() {
            frame_buffer.draw_pixel(x, y, &PixelColor::new(255, 255, 255))
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let _hello = 0;
    loop {}
}
