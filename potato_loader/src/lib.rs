#![no_std]
#![feature(asm)]

pub mod frame_buffer;

use core::arch::asm;
#[inline]
pub fn int3() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}