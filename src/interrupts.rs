

use crate::xhc::XHC_CONTROLLER;
use crate::error;

fn interrupt_handler_xhc(frame: *mut u8) {
    let mut controller = XHC_CONTROLLER.lock();
    let controller = controller.as_mut().unwrap();
    while controller.has_event() {
        if let Err(e) = controller.process_event() {
            error!("{:?}", e);
        }
    }
    notify_end_of_interrupt();

}

fn notify_end_of_interrupt() {
    const EOI_REGISTER: *mut u8 = 0xfee000b0 as *mut u8;
    unsafe { core::ptr::write_volatile(EOI_REGISTER, 0) }
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
        pub fn set_ist(&mut self, val: u8) {
            assert!(val < 1 << 3);
            self.data = *self.data.set_bits(0..3, val as u16);
        }

        pub fn get_type(&self) -> u8 {
            self.data.get_bits(8..12) as u8
        }
        pub fn set_type(&mut self, val: u8) {
            assert!(val < 1 << 4);
            self.data = *self.data.set_bits(8..12, val as u16);
        }

        pub fn get_dpl(&self) -> u8 {
            self.data.get_bits(13..15) as u8
        }
        pub fn set_dpl(&mut self, val: u8) {
            assert!(val < 1 << 2);
            self.data = *self.data.set_bits(13..15, val as u16);
        }

        pub fn get_present(&self) -> bool {
            self.data.get_bit(15)
        }
        pub fn set_present(&mut self, val: bool) {
            self.data = *self.data.set_bit(15, val);
        }
    }

    impl Default for InterruptDescriptorAttribute {
        fn default() -> Self {
            let mut idta = Self { data: 0 };
            idta.set_present(true); // 1
            idta
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

        pub fn get_offset(&self) -> u64 {
            *0_u64
                .set_bits(0..16, self.offset_low as u64)
                .set_bits(16..32, self.offset_middle as u64)
                .set_bits(32..64, self.offset_high as u64)
        }
        pub fn set_offset(&mut self, offset: u64) {
            let low = offset.get_bits(0..16) as u16;
            let middle = offset.get_bits(16..32) as u16;
            let high = offset.get_bits(32..64) as u32;
            self.offset_low = low;
            self.offset_middle = middle;
            self.offset_high = high;
        }

        pub fn get_ss(&self) -> u16 {
            self.segment_selector
        }
        pub fn set_ss(&mut self, val: u16) {
            self.segment_selector = val;
        }

        pub fn get_attribute(&self) -> InterruptDescriptorAttribute {
            self.attribute
        }
        pub fn set_attribute(&mut self, attr: InterruptDescriptorAttribute) {
            self.attribute = attr;
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

        pub fn set_descriptor(&mut self, iv: u8, desc: InterruptDescriptor) {
            self.data[iv as usize] = desc;
        }
    }

    use crate::sync::SpinMutex;
    pub static IDT: SpinMutex<InterruptDescriptorTable> = SpinMutex::new(InterruptDescriptorTable::missing());
}

