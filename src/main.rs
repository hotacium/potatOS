#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use potatOS::graphics::{
    FrameBuffer, 
    PixelColor, 
    PixelWriter,
    init_global_writer,
};
use potatOS::{kprintln, debug, trace};
use potatOS::mouse::{mouse_observer, init_mouse};
use potatOS::pci::{
    self,
    scan_all_bus,
    Device,
};
use potatOS::xhc::{XHC_CONTROLLER, init_xhc};
use potatOS::logger::set_log_level;
use mikanos_usb as usb;
use core::arch::asm;


fn init() {
    set_log_level(LogLevel::Error);
    init_global_writer(frame_buffer);
    init_mouse();
    // potatOS::interrupts::init_idt();
    scan_all_bus().unwrap();
    init_xhc();
    kprintln!("Welcome to potatOS!");
    trace!("finished initialization");
}

#[no_mangle]
pub extern "C" fn kernel_main(frame_buffer: FrameBuffer) -> ! { // TODO: 引数を参照にする. 8 byte を超える値は参照渡しにすべき.
    for x in 0..frame_buffer.h() {
        for y in 0..frame_buffer.v() {
            frame_buffer.draw_pixel(x, y, &PixelColor::new(255, 255, 255))
        }
    }

    // init 
    init();
    // end init


    loop { XHC_CONTROLLER.lock().as_mut().unwrap().process_event().unwrap(); }

    loop {
        x86_64::instructions::hlt();
    }

}

#[allow(unused)]
unsafe fn divide_by_zero() {
    // asm 
    // ref: https://os.phil-opp.com/catching-exceptions/#inline-assembly
    asm!(
        "div dx",
        inout("dx") 0 => _,
        lateout("ax") _,
        options(nostack),
    );
    kprintln!("SHOULD PANIC");
}
