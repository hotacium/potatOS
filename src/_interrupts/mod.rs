
// pub mod idt;



use x86_64::structures::idt::{
    InterruptDescriptorTable,
    InterruptStackFrame,
    PageFaultErrorCode,
};
// use core::mem::MaybeUninit;
// use crate::kprintln;

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init_idt() {
    unsafe { 
        IDT.divide_error.set_handler_fn(divide_error_handler);
        IDT.debug.set_handler_fn(debug_handler);
        IDT.non_maskable_interrupt.set_handler_fn(non_maskable_interrupt_handler);
        IDT.breakpoint.set_handler_fn(breakpoint_handler); 
        IDT.overflow.set_handler_fn(overflow_handler);
        IDT.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        IDT.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        IDT.device_not_available.set_handler_fn(device_not_available_handler);
        IDT.double_fault.set_handler_fn(double_fault_handler);
        IDT.invalid_tss.set_handler_fn(invalid_tss_handler);
        IDT.segment_not_present.set_handler_fn(segment_not_present_handler);
        IDT.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        IDT.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        IDT.page_fault.set_handler_fn(page_fault_handler);
        IDT.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        IDT.alignment_check.set_handler_fn(alignment_check_handler);
        IDT.machine_check.set_handler_fn(machine_check_handler);
        IDT.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        IDT.virtualization.set_handler_fn(virtualization_handler);
        IDT.security_exception.set_handler_fn(security_exception_handler);
        IDT.load();
    }
}

macro_rules! handler_fn {
    ($name:ident, $message:expr) => {
        extern "x86-interrupt" fn $name(
            stack_frame: InterruptStackFrame,
        ) {
            panic!("{}:\nStack Frame: \n{:#?}", $message, stack_frame);
            // kprintln!("{}:", $message);
            // kprintln!("Stack Frame: \n{:#?}", stack_frame);
        }
    }
}
macro_rules! handler_fn_with_error_code {
    ($name:ident, $message:expr) => {
        extern "x86-interrupt" fn $name(
            stack_frame: InterruptStackFrame,
            error_code: u64,
        ) {
            panic!("{}:\nError Code: {}\nStack Frame: \n{:#?}", $message, error_code, stack_frame);
            // kprintln!("{}:", $message);
            // kprintln!("Error Code: {}", error_code);
            // kprintln!("Stack Frame: \n{:#?}", stack_frame);
        }
    }
}


handler_fn!(divide_error_handler, "DIVIDE ERROR");
handler_fn!(debug_handler, "DEBUG");
handler_fn!(non_maskable_interrupt_handler, "HandlerFunc");
handler_fn!(breakpoint_handler, "BREAKPOINT");
handler_fn!(overflow_handler, "OVERFLOW");
handler_fn!(bound_range_exceeded_handler, "BOUND RANGE EXCEEDED HANDLER");
handler_fn!(invalid_opcode_handler, "INVALID OPCODE");
handler_fn!(device_not_available_handler, "DEVICE NOT AVAILABLE");
// -> double_fault_handler
handler_fn_with_error_code!(invalid_tss_handler, "INVALID TSS");
handler_fn_with_error_code!(segment_not_present_handler, "SEGMENT NOT PRESENT");
handler_fn_with_error_code!(stack_segment_fault_handler, "STACK SEGMENT FAULT");
handler_fn_with_error_code!(general_protection_fault_handler, "GENERAL PROTECTION FAULT");
// -> page_fault_handler
handler_fn!(x87_floating_point_handler, "X87 FLOATING POINT");
handler_fn_with_error_code!(alignment_check_handler, "ALIGNMENT CHECK");
// -> machine_check_handler
handler_fn!(simd_floating_point_handler, "SIMD FLOATING POINT");
handler_fn!(virtualization_handler, "VIRTUALIZATION");
handler_fn_with_error_code!(security_exception_handler, "SECURITY EXCEPTION");


extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    panic!("PAGE FAULT");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    // loop { x86_64::instructions::hlt() }
}

extern "x86-interrupt" fn machine_check_handler (
    stack_frame: InterruptStackFrame,
) -> ! {
    panic!("EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}
