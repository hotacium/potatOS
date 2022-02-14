
use crate::sync::SpinMutex;
use crate::pci::{self, Device};
use crate::trace;
use mikanos_usb as usb;


pub static XHC_CONTROLLER: SpinMutex<Option<&'static mut usb::xhci::Controller>> 
    = SpinMutex::new(None); // MaybeUninit, Option, 

pub fn init_xhc() {
    let xhc_dev = find_xhc_device();
    if let Some(device) = xhc_dev {
        let xhc_bar = device.read_bar(0);
        let mmio_base = (xhc_bar.unwrap() & !0x0f) as u64;
        let mut controller = XHC_CONTROLLER.lock();
        *controller = Some(unsafe { mikanos_usb::xhci::Controller::new(mmio_base) });
        let controller = controller.as_mut().unwrap();

        if device.as_config().read_vendor_id() == 0x8086 {
            switch_echi_to_xhci(pci::devices(), device);
        }

        controller.init();
        trace!("xhc initialized");
        controller.run().unwrap();

        use crate::mouse::mouse_observer;
        usb::HidMouseDriver::set_default_observer(mouse_observer);
        controller.configure_connected_ports();

        // loop { controller.process_event().unwrap(); }
    }

}


fn find_xhc_device() -> Option<&'static Device> {
    let mut xhc_dev = None;
    for device in pci::devices() {
        let config = device.as_config();
        trace!("{:?}", config.read_class_code());
        if let (0x0c, 0x03, 0x30, _) = config.read_class_code() {
            trace!("detected xhc device");
            xhc_dev = Some(device);
            
            if config.read_vendor_id() == 0x8086 {
                break
            }
        }
    }
    xhc_dev
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















