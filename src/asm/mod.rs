
//! 参考: https://github.com/rust-osdev/x86_64
//! 
//! 


use core::arch::asm;
use crate::interrupts::idt::InterruptDescriptorTablePointer;


#[inline]
pub fn int3() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}

#[inline]
pub fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

#[inline]
pub fn get_cs() -> u16 {
    let cs: u16;
    unsafe {
        asm!("mov {:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags))
    }
    cs
}

#[inline]
pub fn lidt(idt: &InterruptDescriptorTablePointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }
}

