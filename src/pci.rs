use core::arch::asm;
use crate::interrupts::idt::InterruptVector;

type Result<T> = core::result::Result<T, ()>;


const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Clone, Copy)]
pub struct Config {
    bus: u8,
    device: u8,
    function: u8,
}

use crate::utils::bit_field::BitField;
impl Config {
    pub fn make_address(&self, reg_addr: u8) -> u32 {
        let mut value = 0;
        value = *value.set_bit(31, true)
            .set_bits(24..31, 0)
            .set_bits(16..24, self.bus as u32)
            .set_bits(11..16, self.device as u32)
            .set_bits(8..11, self.function as u32)
            .set_bits(0..8, reg_addr as u32 & 0xfc);
        value
    }

    pub fn read_vendor_id(&self) -> u16 {
        write_config_addr(self.make_address(0x0));
        read_config_data()
            .get_bits(0..16) as u16
    }

    pub fn read_device_id(&self) -> u16 {
        write_config_addr(self.make_address(0x0));
        read_config_data()
            .get_bits(16..32) as u16
    }

    pub fn read_class_code(&self) -> (u8, u8, u8, u8) {
        let class_code = {
            write_config_addr(self.make_address(0x08));
            read_config_data()
        };
        (
            class_code.get_bits(24..32) as u8,
            class_code.get_bits(16..24) as u8,
            class_code.get_bits(8..16) as u8,
            class_code.get_bits(0..8) as u8,
        )
    }

    pub fn read_header_type(&self) -> u8 {
        write_config_addr(self.make_address(0x0c));
        read_config_data()
            .get_bits(16..24) as u8
    }

    pub fn read_bus_number(&self) -> u32 {
        write_config_addr(self.make_address(0x18));
        read_config_data()
    }


}


use crate::utils::fixed_vec::FixedVec;
const MAX_DEVICES_NUM: usize = 32;
pub static mut DEVICES: FixedVec<Device, MAX_DEVICES_NUM> = FixedVec::new();

pub fn devices() -> &'static [Device] {
    unsafe { DEVICES.as_slice() }
}

pub struct Device {
    bus: u8,
    device: u8,
    function: u8,
    header_type: u8,
}

impl From<Config> for Device {
    fn from(config: Config) -> Self {
        let Config {
            bus, device, function,
        } = config;
        Self {
            bus,
            device,
            function,
            header_type: config.read_header_type()
        }
    }
}

use core::fmt;
impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vendor_id = self.as_config().read_vendor_id();
        let (base, sub, iface, rev_id) = self.as_config().read_class_code();
        let class_code = *0u32
                .set_bits(24..32, base as u32)
                .set_bits(16..24, sub as u32)
                .set_bits(8..16, iface as u32)
                .set_bits(0..8, rev_id as u32);
        write!(f, 
            "{}.{}.{}: vendor {:x}, class: {:x}, head: {:x}", 
            self.bus, self.device, self.function, vendor_id, class_code, self.header_type
        )
    }
}

impl Device {
    pub fn as_config(&self) -> Config {
        Config {
            bus: self.bus,
            device: self.device,
            function: self.function,
        }
    }

    pub fn read_register(&self, reg_idx: u8) -> u32 {
        let addr = self.as_config().make_address(reg_idx);
        write_config_addr(addr);
        read_config_data()
    }

    pub fn write_register(&self, reg_idx: u8, val: u32) {
        let addr = self.as_config().make_address(reg_idx);
        write_config_addr(addr);
        write_config_data(val);
    }

    pub fn read_bar(&self, bar_idx: u8) -> Option<u64> {
        if bar_idx >= 6 {
            return None;
        }

        let addr = bar_idx*4 + 0x10;
        let bar_lower = self.read_register(addr) as u64;

        if bar_lower & 0b100 == 0  {
            return Some(bar_lower);
        }

        if bar_idx >= 5 {
            return None;
        }

        let bar_upper = self.read_register(addr+4) as u64;

        let bar = bar_upper << 32 | bar_lower;
        Some(bar)
    }

    pub fn configure_msi_fixed_destination(
        &self, 
        apic_id: u8, 
        trigger_mode: MSITriggerMode,
        derivary_mode: MSIDeliveryMode,
        vector: InterruptVector,
        num_vector_exponent: u32,
    ) -> Result<()> {
        let msg_addr: u32 = *0xfee00000.set_bits(12..20, apic_id as u32);

        let mut msg_data = *0_u32
            .set_bits(8..11, derivary_mode as u32)
            .set_bits(0..8, vector as u32);
        if trigger_mode == MSITriggerMode::Level {
            msg_data |= 0xc000; // trigger mode, level を 1 をセット
        }
        
        self.configure_msi(msg_addr, msg_data, num_vector_exponent)
    }

    fn configure_msi(
        &self, 
        msg_addr: u32, 
        msg_data: u32, 
        num_vector_exponent: u32
    ) -> Result<()> {
        // 最初の capability pointer を読む (32bit から下位 8bit のみ必要)
        let mut cap_addr = self.read_register(0x34).get_bits(0..8) as u8;
        let (mut msi_cap_addr, mut _msix_cap_addr) = (0_u8, 0_u8);

        // msi の場所を探索 (pci コンフィグレーション空間から capability pointer をたどる)
        while cap_addr != 0 {
            let header = CapabilityHeader {
                data: self.read_register(cap_addr),
            };
            match header.cap_id() {
                CapabilityHeader::CAPABILITY_ID_MSI => msi_cap_addr = cap_addr,
                CapabilityHeader::CAPABILITY_ID_MSIX => _msix_cap_addr = cap_addr,
                _ => {}
            }
            cap_addr = header.next_ptr();
        }

        // crate::kprintln!("msi_cap_addr: {:x}", msi_cap_addr);
        if msi_cap_addr != 0 {
            self.configure_msi_register(msi_cap_addr, msg_addr, msg_data, num_vector_exponent);
            Ok(())
        } else {
            Err(())
        }
    }

    fn configure_msi_register(
        &self, 
        cap_addr: u8, 
        msg_addr: u32, 
        msg_data: u32, 
        num_vector_exponent: u32
    ) {
        let mut msi_cap = self.read_msi_capability(cap_addr);
        let multi_message_capable = msi_cap.header.get_multi_message_capable();
        let multi_message_enable = num_vector_exponent.min(multi_message_capable as u32) as u8;
        msi_cap.header.set_multi_message_enable(multi_message_enable);
        msi_cap.header.set_msi_enable(true);
        msi_cap.msg_addr = msg_addr;
        msi_cap.msg_data = msg_data;

        self.write_msi_capability(cap_addr, &msi_cap);
    }

    fn read_msi_capability_header(&self, cap_addr: u8) -> CapabilityHeader {
        CapabilityHeader {
            data: self.read_register(cap_addr),
        }
    }

    fn read_msi_capability(&self, cap_addr: u8) -> MSICapability {
        let header = self.read_msi_capability_header(cap_addr);
        let msg_addr = self.read_register(cap_addr + 4);
        let (msg_upper_addr, msg_data_addr) = if header.get_64_bit_address_capable() {
            (self.read_register(cap_addr + 8), cap_addr + 12)
        } else {
            (0, cap_addr + 8)
        };
        let msg_data = self.read_register(msg_data_addr);
        let (mask_bits, pending_bits) = if header.get_per_vector_masking_capable() {
            (
                self.read_register(msg_data_addr + 4),
                self.read_register(msg_data_addr + 8),
            )
        } else {
            (0, 0)
        };

        MSICapability {
            header,
            msg_addr,
            msg_upper_addr,
            msg_data,
            mask_bits,
            pending_bits,
        }
    }

    fn write_msi_capability(&self, cap_addr: u8, msi_cap: &MSICapability) {
        let header_data = msi_cap.header.as_u32(); 
        self.write_register(cap_addr, header_data);
        self.write_register(cap_addr + 4, msi_cap.msg_addr);

        let msg_data_addr = if msi_cap.header.get_64_bit_address_capable() {
            self.write_register(cap_addr + 8, msi_cap.msg_upper_addr);
            cap_addr + 12
        } else {
            cap_addr + 8
        };
        self.write_register(msg_data_addr, msi_cap.msg_data);

        if msi_cap.header.get_per_vector_masking_capable() {
            self.write_register(msg_data_addr + 4, msi_cap.mask_bits);
            self.write_register(msg_data_addr + 8, msi_cap.pending_bits);
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct CapabilityHeader {
    data: u32,
}

impl CapabilityHeader {
    const CAPABILITY_ID_MSI: u8 = 0x05;
    const CAPABILITY_ID_MSIX: u8 = 0x11;

    pub fn cap_id(&self) -> u8 {
        self.data.get_bits(0..8) as u8
    }

    pub fn next_ptr(&self) -> u8 {
        self.data.get_bits(8..16) as u8
    }

    pub fn get_per_vector_masking_capable(&self) -> bool {
        // offset + 8
        self.data.get_bit(24)
    }
    pub fn get_64_bit_address_capable(&self) -> bool {
        // offset + 7
        self.data.get_bit(23)
    }

    pub fn set_multi_message_enable(&mut self, val: u8) {
        // assert!(val < 1 << 2);
        // offset + 4 ... offeset + 7
        self.data = *self.data.set_bits(20..23, val as u32);
    }

    pub fn get_multi_message_capable(&self) -> u8 {
        // offset + 1 ... offset + 4
        self.data.get_bits(17..20) as u8
    }

    pub fn set_msi_enable(&mut self, val: bool) {
        // offset + 0
        self.data = *self.data.set_bit(16, val);
    }

    pub fn as_u32(&self) -> u32 {
        self.data
    }
}

#[derive(Debug)]
#[repr(C)]
struct MSICapability {
    pub header: CapabilityHeader,
    pub msg_addr: u32, //
    pub msg_upper_addr: u32, //
    pub msg_data: u32, // 
    pub mask_bits: u32, // 
    pub pending_bits: u32, // 
}

#[derive(PartialEq, Debug)]
#[repr(u8)]
pub enum MSITriggerMode {
    Edge = 0,
    Level = 1,
}

#[derive(Debug)]
#[repr(u8)]
pub enum MSIDeliveryMode {
    Fixed = 0b000,
    LowestPriority = 0b001,
    SMI = 0b010,
    NMI = 0b100,
    INIT = 0b101,
    ExtINT = 0b111,
}

pub fn write_config_addr(addr: u32) {
    let mut port = IOPort::new(CONFIG_ADDRESS);
    port.write32(addr);
}
pub fn write_config_data(data: u32) {
    let mut port = IOPort::new(CONFIG_DATA);
    port.write32(data);
}
pub fn read_config_data() -> u32 {
    let mut port = IOPort::new(CONFIG_DATA);
    port.read32()
}

fn is_single_function_device(header_type: u8) -> bool {
    !header_type.get_bit(7)
}

pub fn scan_all_bus() -> Result<()> {
    // 探索の起点
    let host_bridge = Config {
        bus: 0,
        device: 0,
        function: 0,
    };
    // true なら host_bridge が バス0 のホストブリッジで, 
    if is_single_function_device(host_bridge.read_header_type()) {
        return scan_bus(host_bridge);
    }
    for function in 1..8 {
        let another_host_bridge = Config {
            bus: 0,
            device: 0,
            function,
        };
        // 0xffff ならバスが存在しない
        if another_host_bridge.read_vendor_id() == 0xffff {
            continue;
        }
        scan_bus(Config {
            bus: function,
            device: 0,
            function: 0,
        })?;
    }
    Ok(())
}

fn scan_bus(config: Config) -> Result<()> {
    for device in 0..32 {
        let dev = Config {
            bus: config.bus,
            device,
            function: 0,
        };
        if dev.read_vendor_id() == 0xffff {
            continue;
        }
        scan_device(dev)?;
    }
    Ok(())
}

fn scan_device(mut config: Config) -> Result<()> {
    config.function = 0;
    scan_function(config)?;
    if is_single_function_device(config.read_header_type()) {
        return Ok(());
    }
    for function in 1..8 {
        config.function = function;
        if config.read_vendor_id() == 0xffff {
            continue;
        }
        scan_function(config)?;
    }
    Ok(())
}

fn scan_function(config: Config) -> Result<()> {
    add_device(config)?;
    let (base, sub, _, _) = config.read_class_code();
    // PCI-to-PCI device
    if base == 0x06 && sub == 0x04 {
        let bus_number = config.read_bus_number();
        let secondary_bus = bus_number.get_bits(8..16) as u8;
        scan_bus(Config {
            bus: secondary_bus,
            device: 0,
            function: 0,
        })
    } else {
        Ok(())
    }
}

fn add_device(config: Config) -> Result<()> {
    let device: Device = config.into();
    if unsafe { DEVICES.try_push(device).is_ok() } {
        Ok(())
    } else {
        Err(())
    }
}

pub struct IOPort {
    port: u16,
}

impl IOPort {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn read16(&mut self) -> u16 {
        let eax: u16;
        unsafe { asm!(
            "in eax, dx",
            out("eax") eax,
            in("dx") self.port,
        ) };
        eax
    }

    pub fn read32(&mut self) -> u32 {
        let eax: u32;
        unsafe { asm!(
            "in eax, dx",
            out("eax") eax,
            in("dx") self.port,
        ) };
        eax
    }

    pub fn write16(&mut self, data: u16) {
        unsafe { asm!(
            "out dx, eax",
            in("dx") self.port,
            in("eax") data,
        ) };
    }

    pub fn write32(&mut self, data: u32) {
        unsafe { asm!(
            "out dx, eax",
            in("dx") self.port,
            in("eax") data,
        ) };
    }

}
