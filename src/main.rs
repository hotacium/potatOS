#![no_std]
#![no_main]
#![feature(asm)]
#![feature(core_intrinsics)]

use core::panic::PanicInfo;
use potatOS::graphics::{
    FrameBuffer, 
    PixelColor, 
    PixelWriter,
    PixelFormat, 
    WRITER,
    RGBResv8BitPerColorPixelWriter,
    BGRResv8BitPerColorPixelWriter,
};
use potatOS::kprintln;
use potatOS::mouse::MOUSE_CURSOR_SHAPE;
use potatOS::pci::{
    self,
    scan_all_bus,
    Device,
};
use mikanos_usb as usb;

#[no_mangle]
pub extern "C" fn kernel_main(frame_buffer: FrameBuffer) -> ! { // TODO: 引数を参照にする. 8 byte を超える値は参照渡しにすべき.
    for x in 0..frame_buffer.h() {
        for y in 0..frame_buffer.v() {
            frame_buffer.draw_pixel(x, y, &PixelColor::new(255, 255, 255))
        }
    }
    
    // init 
    use core::mem::MaybeUninit;
    WRITER.lock().write(match frame_buffer.pixel_format() {
        PixelFormat::PixelRGBResv8BitPerColor => {
            // placement new
            static mut RGB_WRITER: MaybeUninit<RGBResv8BitPerColorPixelWriter> = MaybeUninit::uninit();
            unsafe { RGB_WRITER.write(RGBResv8BitPerColorPixelWriter::new(frame_buffer)); }
            unsafe { RGB_WRITER.assume_init_ref() }
        },
        PixelFormat::PixelBGRResv8BitPerColor => {
            static mut BGR_WRITER: MaybeUninit<BGRResv8BitPerColorPixelWriter> = MaybeUninit::uninit();
            unsafe { BGR_WRITER.write(BGRResv8BitPerColorPixelWriter::new(frame_buffer)); }
            unsafe { BGR_WRITER.assume_init_ref() }
        },
    });
    

    // WRITER.lock().init(frame_buffer);
    // to avoid deadlock between `writer` and `println!`
    // (both of them use same static WRITER with spinlock)
    {  
        let writer = unsafe { WRITER.lock().assume_init() };
        for dy in 0..MOUSE_CURSOR_SHAPE.len() {
            MOUSE_CURSOR_SHAPE[dy].chars()
                .enumerate()
                .for_each(|(dx, c)| {
                    match c {
                        '@' => writer.draw_pixel(200+dx, 100+dy, &PixelColor::BLACK),
                        '.' => writer.draw_pixel(200+dx, 100+dy, &PixelColor::WHITE),
                        ' ' => { /* do nothing */ },
                        c => panic!("Unexpected cursor shape: {}", c),
                    }
            });
        }
    } // unlock WRITER.lock()

    // unsafe { divide_by_zero() };

    scan_all_bus().unwrap();
    for device in pci::devices() {
        // kprintln!("{:?}", device);
    }

    let mut xhc_dev: Option<&pci::Device> = None;
    for device in pci::devices() {
        let config = device.as_config();
        // kprintln!("{:?}", config.read_class_code());
        if let (0x0c, 0x03, 0x30, _) = config.read_class_code() {
            kprintln!("detected xhc device");
            xhc_dev = Some(device);
            
            if config.read_vendor_id() == 0x8086 {
                break
            }
        }
    }
    
    if let Some(device) = xhc_dev {
        let xhc_bar = device.read_bar(0);
        kprintln!("xhc bar: {:08x}", xhc_bar.unwrap());
        let mmio_base = (xhc_bar.unwrap() & !0x0f) as u64;
        kprintln!("mmio_base: {:08x}", mmio_base);
        let controller = unsafe { mikanos_usb::xhci::Controller::new(mmio_base) };

        if device.as_config().read_vendor_id() == 0x8086 {
            switch_echi_to_xhci(pci::devices(), device);
        }

        controller.init();
        kprintln!("xhc initialized");
        controller.run().unwrap();

        // usb::HidMouseDriver::set_default_observer()
    }

    kprintln!("Welcome to potatOS!");
    // kprintln!("1+2={:?}", 1+2);

    loop {
        unsafe { asm!("hlt") };
    }
}

fn switch_echi_to_xhci(devices: &[Device], xhc_dev: &Device) {
    let has_intel_ehc = devices.iter().any(|device| {
        let conf = device.as_config();
        let code = conf.read_class_code();
        (0x0c, 0x03, 0x20) == (code.0, code.1, code.2) && conf.read_vendor_id() == 0x8086
    });

    if !has_intel_ehc {
        return;
    }
    let superspeed_ports = xhc_dev.read_register(0xdc);
    xhc_dev.write_register(0xf8, superspeed_ports);
    let ehci2xhci_ports = xhc_dev.read_register(0xd4);
    xhc_dev.write_register(0xd0, ehci2xhci_ports);
}


// TODO: write another panic function for release build
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // unsafe { core::intrinsics::breakpoint(); } // for GDB debugging // only for x86
    if let Some(loc) = _info.location() {
        // below is for GDB debugging
        let (_file, _line) = (loc.file(), loc.line());
    }
    loop {}
}

unsafe fn divide_by_zero() {
    kprintln!("DIVIDE BY ZERO");
    // asm 
    // ref: https://os.phil-opp.com/catching-exceptions/#inline-assembly
    asm!(
        "div dx",
        inout("dx") 0 => _,
        lateout("ax") _,
        options(nostack),
    );
    kprintln!("SHOULD PANIC");
}
