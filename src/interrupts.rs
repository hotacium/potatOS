


mod interrupt_handler {
    use crate::xhc::XHC_CONTROLLER;
    use crate::error;
    use super::idt::InterruptStackFrame;

    pub extern "x86-interrupt" fn xhc_handler(_frame: *mut InterruptStackFrame) {
        panic!("xhc_handler");
        /*
        let mut controller = XHC_CONTROLLER.lock();
        let controller = controller.as_mut().unwrap();
        while controller.has_event() {
            if let Err(e) = controller.process_event() {
                error!("{:?}", e);
            }
        }
        */
        notify_end_of_interrupt();
    }

    fn notify_end_of_interrupt() {
        const EOI_REGISTER: *mut u8 = 0xfee000b0 as *mut u8;
        unsafe { core::ptr::write_volatile(EOI_REGISTER, 0) }
    }

    pub extern "x86-interrupt" fn divide_by_zero_handler(_frame: *mut InterruptStackFrame) {
        panic!("divide by zero");
    }

    pub extern "x86-interrupt" fn breakpoint_handler(_frame: *mut InterruptStackFrame) {
        crate::kprintln!("breakpoint");
    }

    pub extern "x86-interrupt" fn double_fault_handler(_frame: *mut InterruptStackFrame, _error_code: u64) {
        panic!("double fault");
    }

    pub extern "x86-interrupt" fn invalid_tss_handler(_frame: *mut InterruptStackFrame, _error_code: u64) {
        panic!("invalid tss");
    }

    pub extern "x86-interrupt" fn segment_not_present_handler(_frame: *mut InterruptStackFrame, _error_code: u64) {
        panic!("segment not present");
 
    }

    pub extern "x86-interrupt" fn stack_segment_fault_handler(_frame: *mut InterruptStackFrame, _error_code: u64) {
        panic!("stack segment fault");
    }

    pub extern "x86-interrupt" fn general_protection_fault_handler(_frame: *mut InterruptStackFrame, _error_code: u64) {
        panic!("general protection fault");
    }

    pub extern "x86-interrupt" fn page_fault_handler(_frame: *mut InterruptStackFrame, _error_code: u64) {
        panic!("page fault");
    }

}

pub mod idt {
    use crate::utils::bit_field::BitField;

    #[derive(Debug, Clone, Copy)]
    #[repr(transparent)]
    pub struct InterruptDescriptorAttribute {
        data: u16
    }

    impl InterruptDescriptorAttribute {

        pub const fn missing() -> Self {
            Self { data: 0 }
        }

        pub fn get_ist(&self) -> u8 {
            self.data.get_bits(0..3) as u8
        }

        #[must_use]
        pub fn set_ist(mut self, val: u8) -> Self {
            assert!(val < 1 << 3);
            self.data = *self.data.set_bits(0..3, val as u16);
            self
        }

        pub fn get_type(&self) -> u8 {
            self.data.get_bits(8..12) as u8
        }

        #[must_use]
        pub fn set_type(mut self, val: u8) -> Self {
            assert!(val < 1 << 4);
            self.data = *self.data.set_bits(8..12, val as u16);
            self
        }

        pub fn get_dpl(&self) -> u8 {
            self.data.get_bits(13..15) as u8
        }

        #[must_use]
        pub fn set_dpl(mut self, val: u8) -> Self {
            assert!(val < 1 << 2);
            self.data = *self.data.set_bits(13..15, val as u16);
            self
        }

        pub fn get_present(&self) -> bool {
            self.data.get_bit(15)
        }

        #[must_use]
        pub fn set_present(mut self, val: bool) -> Self {
            self.data = *self.data.set_bit(15, val);
            self
        }
    }

    impl Default for InterruptDescriptorAttribute {
        fn default() -> Self {
            Self { data: 0 }
                .set_present(true)
        }
    }

    #[derive(Debug, Default, Clone, Copy)]
    #[repr(C)]
    pub struct InterruptDescriptor {
        offset_low: u16,
        segment_selector: u16,
        attribute: InterruptDescriptorAttribute, // u16
        offset_middle: u16,
        offset_high: u32,
        _reserved: u32,
    }

    impl InterruptDescriptor {
        pub const fn missing() -> Self {
            Self {
                offset_low: 0,
                segment_selector: 0,
                attribute: InterruptDescriptorAttribute::missing(),
                offset_middle: 0,
                offset_high: 0,
                _reserved: 0,
            }
        }

        pub fn new(handler_ptr: u64, attr: InterruptDescriptorAttribute) -> Self {
            let cs = crate::asm::get_cs();
            Self {
                offset_low: handler_ptr.get_bits(0..16) as u16,
                segment_selector: cs,
                attribute: attr,
                offset_middle: handler_ptr.get_bits(16..32) as u16,
                offset_high: handler_ptr.get_bits(32..64) as u32,
                _reserved: 0,
            }
        }

        pub fn get_offset(&self) -> u64 {
            *0_u64
                .set_bits(0..16, self.offset_low as u64)
                .set_bits(16..32, self.offset_middle as u64)
                .set_bits(32..64, self.offset_high as u64)
        }

        #[must_use]
        pub fn set_offset(mut self, offset: u64) -> Self {
            self.offset_low = offset.get_bits(0..16) as u16;
            self.offset_middle = offset.get_bits(16..32) as u16;
            self.offset_high = offset.get_bits(32..64) as u32;
            self
        }

        pub fn get_ss(&self) -> u16 {
            self.segment_selector
        }

        #[must_use]
        pub fn set_ss(mut self, val: u16) -> Self {
            self.segment_selector = val;
            self
        }

        pub fn get_attribute(&self) -> InterruptDescriptorAttribute {
            self.attribute
        }

        #[must_use]
        pub fn set_attribute(mut self, attr: InterruptDescriptorAttribute) -> Self {
            self.attribute = attr;
            self
        }
    }

    #[derive(Debug)]
    #[repr(transparent)]
    pub struct InterruptDescriptorTable {
        data: [InterruptDescriptor; 256]
    }

    impl InterruptDescriptorTable {
        pub const fn missing() -> Self {
            Self { data: [InterruptDescriptor::missing(); 256] }
        }

        pub fn set_handler(&mut self, iv: u8, handler_ptr: u64, attr: InterruptDescriptorAttribute) {
            self.set_descriptor(iv, InterruptDescriptor::new(handler_ptr, attr));
        }
        pub fn set_descriptor(&mut self, iv: u8, desc: InterruptDescriptor) {
            self.data[iv as usize] = desc;
        }

        pub fn as_ptr(&self) -> InterruptDescriptorTablePointer {
            use core::mem::size_of;
            InterruptDescriptorTablePointer { 
               limit: (size_of::<Self>() -1) as u16, 
               offset: self as *const Self as u64, 
               _reserved: [0; 5],
            }
        }

        pub fn load(&self) {
            self.as_ptr().load()
        }
    }

    use crate::sync::SpinMutex;
    pub static IDT: SpinMutex<InterruptDescriptorTable> = SpinMutex::new(InterruptDescriptorTable::missing());

    // IDT を初期化し, ロードする
    // これは, 初期化時に一度だけ呼び出すこと
    pub fn init_idt() {
        // type InterurptHandler = extern "x86-interrupt" fn(*mut u8); // TODO: *mut u8 を *mut InterruptStackFrame に変更する

        let mut idt = IDT.lock();
        idt.set_handler(
            InterruptVector::XHCI as u8, 
            super::interrupt_handler::xhc_handler as usize as u64, 
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        idt.set_handler(
            InterruptVector::DivideByZeroError as u8,
            super::interrupt_handler::divide_by_zero_handler as usize as u64,
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        idt.set_handler(
            InterruptVector::Breakpoint as u8, 
            super::interrupt_handler::breakpoint_handler as usize as u64, 
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        idt.set_handler(
            InterruptVector::DoubleFault as u8,
            super::interrupt_handler::double_fault_handler as usize as u64,
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        idt.set_handler(
            InterruptVector::InvalidTss as u8,
            super::interrupt_handler::invalid_tss_handler as usize as u64,
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        idt.set_handler(
            InterruptVector::SegmentNotPresent as u8,
            super::interrupt_handler::segment_not_present_handler as usize as u64,
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        idt.set_handler(
            InterruptVector::Stack as u8,
            super::interrupt_handler::stack_segment_fault_handler as usize as u64,
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        idt.set_handler(
            InterruptVector::GeneralProtection as u8,
            super::interrupt_handler::general_protection_fault_handler as usize as u64,
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        idt.set_handler(
            InterruptVector::PageFault as u8,
            super::interrupt_handler::page_fault_handler as usize as u64,
            InterruptDescriptorAttribute::missing()
                .set_type(14) // interrupt gate == 14
                .set_dpl(0) // ring 0
                .set_present(true),
        );
        // idt.set_descriptor(InterruptVector::XHCI as u8, xhc_desc);
        idt.load();
    }

    #[repr(C, packed)]
    pub struct InterruptDescriptorTablePointer {
        limit: u16,
        offset: u64,
        _reserved: [u16; 5],
    }

    impl InterruptDescriptorTablePointer {
        pub fn load(&self) {
            crate::asm::lidt(self);
        }

        #[must_use]
        pub fn set_limit(mut self, limit: u16) -> Self {
            self.limit = limit;
            self
        }

        #[must_use]
        pub fn set_offset(mut self, offset: u64) -> Self {
            self.offset = offset;
            self
        }

    }

    // - https://wiki.osdev.org/Interrupt_Vector_Table
    // - https://www.amd.com/system/files/TechDocs/24593.pdf: Table 8-1. Interrupt Vector Source and Cause
    #[derive(Debug, Clone, Copy)]
    pub enum InterruptVector {
        DivideByZeroError = 0x00,
        Breakpoint = 0x03,
        DoubleFault = 0x08,
        InvalidTss = 0x0A,
        SegmentNotPresent = 0x0B,
        Stack = 0x0C,
        GeneralProtection = 0x0D,
        PageFault = 0x0E,
        XHCI = 0x40,
    }

    #[derive(Debug)]
    pub struct InterruptStackFrame {
        pub rip: u64,
        pub cs: u64,
        pub rflags: u64,
        pub rsp: u64,
        pub ss: u64,
    }
}

