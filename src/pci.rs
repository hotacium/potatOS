
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
        let address = {
            *(0.set_bit(31, true)
                .set_bits(16..24, self.bus as u32)
                .set_bits(11..16, self.device as u32)
                .set_bits(8..11, self.function as u32)
                .set_bits(2..8, reg_addr as u32))
        };
        address
    }

    pub fn read_vendor_id(&self) -> u16 {
        unsafe { 
            self.write_config_addr(self.make_address(0));
            self.read_config_data()
                .get_bits(0..16) as u16
        }
    }

    pub fn read_device_id(&self) -> u16 {
        unsafe {
            self.write_config_addr(self.make_address(0));
            self.read_config_data()
                .get_bits(16..32) as u16
        }
    }

    pub fn read_class_code(&self) -> (u8, u8, u8, u8) {
        let class_code = unsafe {
            self.write_config_addr(self.make_address(2));
            self.read_config_data()
        };
        (
            class_code.get_bits(0..8) as u8,
            class_code.get_bits(8..16) as u8,
            class_code.get_bits(16..24) as u8,
            class_code.get_bits(24..32) as u8,
        )
    }

    pub fn read_header_type(&self) -> u8 {
        unsafe {
            self.write_config_addr(self.make_address(3));
            self.read_config_data()
                .get_bits(16..24) as u8
        }
    }

    pub fn read_bus_number(&self) -> u32 {
        unsafe {
            self.write_config_addr(self.make_address(6));
            self.read_config_data()
        }
    }

    unsafe fn write_config_addr(&self, addr: u32) {
        let mut port = IOPort::new(CONFIG_ADDRESS);
        port.write32(addr);
    }
    unsafe fn write_config_data(&self, data: u32) {
        let mut port = IOPort::new(CONFIG_DATA);
        port.write32(data);
    }
    unsafe fn read_config_data(&self) -> u32 {
        let mut port = IOPort::new(CONFIG_DATA);
        port.read32()
    }
}


use crate::fixed_vec::FixedVec;
const MAX_DEVICES_NUM: usize = 32;
static mut DEVICES: FixedVec<Device, MAX_DEVICES_NUM> = FixedVec::new();

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
    todo!()
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