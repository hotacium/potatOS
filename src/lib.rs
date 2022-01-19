#![no_std]
#![feature(const_maybe_uninit_assume_init)]
#![feature(abi_x86_interrupt)]

pub mod graphics;
pub mod console;
pub mod mouse;
pub mod sync;
pub mod bit_field;
pub mod pci;
pub mod fixed_vec;
pub mod interrupts;

use core::panic::PanicInfo;
// TODO: write another panic function for release build
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// cant trap with qemu
use core::arch::asm;
pub fn int3() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}

pub unsafe fn hlt() {
    asm!("hlt", options(nomem, nostack, preserves_flags)); 
}
