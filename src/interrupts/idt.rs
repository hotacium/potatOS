
use crate::bit_field::BitField;

#[derive(Default, Clone, Copy)]
#[repr(packed)]
pub struct Entry {
    pointer_low: u16,
    gdt_selector: u16,
    options: EntryOptions, // u16
    pointer_middle: u16,
    pointer_high: u32,
    _reserved: u32,
}

impl Entry {
    pub fn new(options: EntryOptions, ptr: u64) -> Self {
        let mut entry = Self::default();
        entry.set_pointer(ptr);
        entry.set_options(options);
        entry
    }

    pub fn missing() -> Self {
        Self::default()
    }

    pub fn get_pointer(&self) -> u64 {
        let mut ptr = 0u64;
        ptr.set_bits(0..16, self.pointer_low as u64);
        ptr.set_bits(48..64, self.pointer_middle as u64);
        ptr.set_bits(64..96, self.pointer_high as u64);
        ptr
    }
    pub fn set_pointer(&mut self, ptr: u64) {
        self.pointer_low = ptr.get_bits(0..16) as u16;
        self.pointer_middle = ptr.get_bits(16..32) as u16;
        self.pointer_high = ptr.get_bits(32..64) as u32;
    }

    pub fn get_gdt_descriptor(&self) -> u16 {
        self.gdt_selector
    }
    pub fn set_gdt_descriptor(&mut self, val: u16) {
        self.gdt_selector = val;
    }

    pub fn get_options(&self) -> EntryOptions {
        self.options
    }
    pub fn set_options(&mut self, opt: EntryOptions) {
        self.options = opt;
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct EntryOptions {
    data: u16
}

impl EntryOptions {
    pub fn new() -> Self {
        let mut opt = Self::default();
        opt.set_present(true);
        opt.disable_interrupt(true);
        opt
    }

    pub fn get_ist_idx(&self) -> u8 {
        self.data.get_bits(0..3) as u8
    }
    pub fn set_ist_idx(&mut self, val: u8) {
        self.data = *self.data.set_bits(0..3, val as u16);
    }

    pub fn is_interrupt_disabled(&self) -> bool {
        self.get_gate_type()
    }
    pub fn disable_interrupt(&mut self, disable: bool) {
        self.set_gate_type(disable);
    }
    fn get_gate_type(&self) -> bool {
        self.data.get_bit(8)
    }
    fn set_gate_type(&mut self, val: bool) {
        self.data = *self.data.set_bit(8, val);
    }

    pub fn get_descriptor_privilege_level(&self) -> u8 {
        self.data.get_bits(13..15) as u8
    }
    pub fn set_descriptor_privilege_level(&mut self, val: u8) {
        assert!(val < 2u8.pow(15-13));
        self.data = *self.data.set_bits(13..15, val as u16);
    }

    pub fn get_present(&self) -> bool {
        self.data.get_bit(15)
    }
    pub fn set_present(&mut self, val: bool) {
        self.data.set_bit(15, val);
    }
}

impl Default for EntryOptions {
    fn default() -> Self {
        let mut init_val = 0u16;
        init_val.set_bits(9..12, 0b111 as u16); // 9..12 must be one
        init_val.set_bit(12, false); // 12 must be zero
        Self {
            data: init_val
        }
    }
}

use core::fmt;
impl fmt::Debug for EntryOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let gate_type = match self.data.get_bit(8) {
            false => "Interrupt Gate",
            true => "Trap Gate",
        };
        // refer: https://os.phil-opp.com/cpu-exceptions/
        f.debug_struct("EntryOptions")
            .field("IST Index", &self.data.get_bits(0..3))
            .field("Reserved", &self.data.get_bits(3..8))
            .field("Gate Type", &gate_type)
            .field("must be one (0b111)", &self.data.get_bits(9..12))
            .field("must be zero", &(self.data.get_bit(12) as u8))
            .field("Descriptor Privilege Level", &self.data.get_bits(13..15))
            .field("Present", &(self.data.get_bit(15) as u8))
            .finish()
    }
}

#[repr(transparent)]
pub struct InterruptDescriptorTable {
    data: [Entry; 256],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        Self {
            data: [Entry::missing(); 256],
        }
    }

    // ptr はハンドラ関数へのポインタ
    // todo: 関数を直接渡せるようにしたい
    // 
    // 現在の idt を更新 update するには現在の idt を読む必要がある. 
    // 現状, set -> load で実際に idt を登録する. このときに set した内容は失われる
    pub fn set(&mut self, vec: u8, options: EntryOptions, ptr: u64) {
        let entry = Entry::new(options, ptr);
        self.data[vec as usize] = entry;
    }

    #[must_use]
    pub fn load(&mut self) {
        // IDT を `lidt` を使ってロードする
        // todo: ロードされた IDT が消えることがあるかを考える
        todo!()
    }
}








