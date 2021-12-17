
type Result<T> = core::result::Result<T, ()>;

const CONFIG_ADDRESS: u16 = 0x0cf8;
const CONFIG_DATA: u16 = 0x0cfc;

#[derive(Clone, Copy)]
pub struct Config {
    bus: u8,
    device: u8,
    function: u8,
}

use crate::bit_field::BitField;
impl Config {
    pub fn make_address(&self, reg_addr: u8) -> u32 {
        let mut value = 0;
        value = *value.set_bit(31, true)
            .set_bits(24..31, 0)
            .set_bits(16..24, self.bus as u32)
            .set_bits(11..16, self.device as u32)
            .set_bits(8..11, self.function as u32)
            .set_bits(2..8, reg_addr as u32);
        value
    }

    pub fn read_vendor_id(&self) -> u16 {
        unsafe { 
            write_config_addr(self.make_address(0));
            read_config_data()
                .get_bits(0..16) as u16
        }
    }

    pub fn read_device_id(&self) -> u16 {
        unsafe {
            write_config_addr(self.make_address(0));
            read_config_data()
                .get_bits(16..32) as u16
        }
    }

    pub fn read_class_code(&self) -> (u8, u8, u8, u8) {
        let class_code = unsafe {
            write_config_addr(self.make_address(2));
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
        unsafe {
            write_config_addr(self.make_address(3));
            read_config_data()
                .get_bits(16..24) as u8
        }
    }

    pub fn read_bus_number(&self) -> u32 {
        unsafe {
            write_config_addr(self.make_address(6));
            read_config_data()
        }
    }

}


use crate::fixed_vec::FixedVec;
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

        // let addr = calc_bar_addr(bar_idx);
        let addr = bar_idx + 4;
        let bar_lower = self.read_register(addr) as u64;

        if bar_lower & 0b100 == 0  {
            return Some(bar_lower);
        }

        if bar_idx >= 5 {
            return None;
        }

        let bar_upper = self.read_register(addr+1) as u64;

        let bar = bar_upper << 32 | bar_lower;
        Some(bar)
    }
}

pub fn calc_bar_addr(bar_idx: u8) -> u8 {
    0x10 + bar_idx * 4
}

pub fn write_config_addr(addr: u32) {
    let mut port = unsafe { IOPort::new(CONFIG_ADDRESS) };
    port.write32(addr);
}
pub fn write_config_data(data: u32) {
    let mut port = unsafe { IOPort::new(CONFIG_DATA) };
    port.write32(data);
}
pub fn read_config_data() -> u32 {
    let mut port = unsafe { IOPort::new(CONFIG_DATA) };
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
    if let Ok(_) = unsafe { DEVICES.try_push(device) } {
        Ok(())
    } else {
        Err(())
    }
}

pub struct IOPort {
    port: u16,
}

impl IOPort {
    pub unsafe fn new(port: u16) -> Self {
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
