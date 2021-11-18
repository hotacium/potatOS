#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
use potatOS::graphics::{
    FrameBuffer, 
    PixelColor, 
    PixelWriter, 
};
use potatOS::console::{
    CONSOLE_WRITER,
};
use potatOS::{
    kprintln
};

#[no_mangle]
pub extern "C" fn kernel_main(frame_buffer: FrameBuffer) -> ! {
    for x in 0..frame_buffer.h() {
        for y in 0..frame_buffer.v() {
            frame_buffer.draw_pixel(x, y, &PixelColor::new(255, 255, 255))
        }
    }
    // init
    CONSOLE_WRITER.lock().init(frame_buffer);

    kprintln!("Welcome to potatOS!");
    kprintln!("1+2={:?}", 1+2);

    loop {}
}


// TODO: write another panic function for release build
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { asm!("int3"); } // for GDB debugging // only for x86
    if let Some(loc) = _info.location() {
        // below is for GDB debugging
        let (_file, _line) = (loc.file(), loc.line());
        todo!()
    }
    loop {}
}
