#![no_std]
#![no_main]

use core::panic::PanicInfo;
use potatOS::graphics::{
    FrameBuffer, 
    PixelColor, 
    PixelWriter, 
    ShinonomeFont,
};

#[no_mangle]
pub extern "C" fn kernel_main(frame_buffer: FrameBuffer) -> ! {
    // println!("Hello, world!");
    for x in 0..frame_buffer.h() {
        for y in 0..frame_buffer.v() {
            frame_buffer.draw_pixel(x, y, &PixelColor::new(255, 255, 255))
        }
    }
    let font = ShinonomeFont::new();
    for c in '!'..='~' {
        let position = (c as usize - '!' as usize) * 10;
        let x = position % frame_buffer.h();
        let y = (position / frame_buffer.h())*20 + 50;
        font.write_ascii(&frame_buffer, x, y, c as char, &PixelColor::new(0, 0, 0));
    }

    font.write_string(&frame_buffer, 10, 100, "Hello, World!", &PixelColor::new(0,0,0));

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let _hello = 0;
    loop {}
}
