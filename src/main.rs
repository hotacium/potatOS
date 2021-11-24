#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;
use potatOS::graphics::{
    FrameBuffer, 
    PixelColor, 
    PixelWriter,
    PixelFormat, 
    WRITER,
    RGBResv8BitPerColorPixelWriter,
    BGRResv8BitPerColorPixelWriter,
};
use potatOS::{
    kprintln
};
use potatOS::mouse::{
    MOUSE_CURSOR_SHAPE,
    MOUSE_CURSOR_WIDTH,
};

#[no_mangle]
pub extern "C" fn kernel_main(frame_buffer: FrameBuffer) -> ! { // TODO: 引数が参照にする (?) 8 byte を超える値は参照渡しにすべき
    for x in 0..frame_buffer.h() {
        for y in 0..frame_buffer.v() {
            frame_buffer.draw_pixel(x, y, &PixelColor::new(255, 255, 255))
        }
    }
    
    // init 
    use core::mem::MaybeUninit;
    WRITER.lock().write(match frame_buffer.pixel_format() {
        PixelFormat::PixelRGBResv8BitPerColor => {
            // use placement-new-like way instead of heap allocation
            static mut RGB_WRITER: MaybeUninit<RGBResv8BitPerColorPixelWriter> = MaybeUninit::uninit();
            unsafe { RGB_WRITER.write(RGBResv8BitPerColorPixelWriter::new(frame_buffer)); }
            unsafe { RGB_WRITER.assume_init_ref() }
        },
        PixelFormat::PixelBGRResv8BitPerColor => {
            static mut BGR_WRITER: MaybeUninit<BGRResv8BitPerColorPixelWriter> = MaybeUninit::uninit();
            unsafe { BGR_WRITER.write(BGRResv8BitPerColorPixelWriter::new(frame_buffer)); }
            unsafe { BGR_WRITER.assume_init_ref() }
        },
    });

    // WRITER.lock().init(frame_buffer);
    // to avoid deadlock between `writer` and `println!`
    // (both of them use same static WRITER with spinlock)
    {  
        let writer = unsafe { WRITER.lock().assume_init() };
        for dy in 0..MOUSE_CURSOR_SHAPE.len() {
            MOUSE_CURSOR_SHAPE[dy].chars()
                .enumerate()
                .for_each(|(dx, c)| {
                    match c {
                        '@' => writer.draw_pixel(200+dx, 100+dy, &PixelColor::BLACK),
                        '.' => writer.draw_pixel(200+dx, 100+dy, &PixelColor::WHITE),
                        ' ' => { /* do nothing */ },
                        c => panic!("Unexpected cursor shape: {}", c),
                    }
            });
        }
    } // unlock WRITER.lock()

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
