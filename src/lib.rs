#![no_std]
#![feature(const_maybe_uninit_assume_init)]
#![feature(abi_x86_interrupt)]

pub mod graphics;
pub mod console;
pub mod mouse;
pub mod sync;
pub mod pci;
pub mod interrupts;
pub mod logger;
pub mod xhc;
pub mod utils;
pub mod asm;

use core::panic::PanicInfo;
// TODO: write another panic function for release build
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}