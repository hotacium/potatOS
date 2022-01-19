#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![feature(alloc_error_handler)]
#![feature(asm)]

use core::alloc::Layout;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::{mem, slice};
use uefi::prelude::ResultExt;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, RegularFile, FileType};
use uefi::{
    proto::media::fs::SimpleFileSystem,
    table::boot::{MemoryDescriptor, MemoryType},
};

use potato_loader::frame_buffer::FrameBuffer;
use uefi::prelude::SystemTable;
use uefi::table::Boot;

// type EntryFn = extern "sysv64" fn(&FrameBuffer);
type EntryFn = extern "sysv64" fn(FrameBuffer);

unsafe fn get_frame_buffer(system_table: &SystemTable<Boot>) -> FrameBuffer {
    let frame_buffer = FrameBuffer::from_system_table(system_table);
    frame_buffer
}

struct FileWriter(RegularFile);
use core::fmt;
impl fmt::Write for FileWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0
            .write(s.as_bytes())
            .expect_success("Failed to write to file");
        Ok(())
    }
}

// -------------------------------------------
// EFI_MAIN
// -------------------------------------------
#[no_mangle]
pub extern "efiapi" fn efi_main(
    image: uefi::Handle,
    mut system_table: uefi::table::SystemTable<uefi::table::Boot>,
) -> uefi::Status {
    // UEFI stdout
    let stdout = system_table.stdout();
    stdout.clear().unwrap_success();
    writeln!(stdout, "Hello Bootloader!").unwrap();

    let image_base = {
        let loaded_image = unsafe { system_table
            .boot_services()
            .handle_protocol::<LoadedImage>(image)
            .unwrap_success()
            .get().as_ref().unwrap() };
        let (addr, _) = loaded_image.info();
        addr as u64
    };

    unsafe {
        uefi::alloc::init(system_table.boot_services());
    }

    // retreive memory map
    let mmap_storage = {
        let max_mmap_size =
            system_table.boot_services().memory_map_size() + 8 * mem::size_of::<MemoryDescriptor>();
        let ptr = system_table
            .boot_services()
            .allocate_pool(MemoryType::LOADER_DATA, max_mmap_size)?
            .unwrap();
        unsafe { slice::from_raw_parts_mut(ptr, max_mmap_size) }
    };

    // writeln!(system_table.stdout(), "mmap_storage").unwrap();
    // ------------------------------------------------------
    // get mmap
    let mmap_buf: &mut [u8] = &mut [0;1024*16];
    assert!(system_table.boot_services().memory_map_size() < mmap_buf.len());
    let (_map_key, desc_iter) = system_table.boot_services()
        .memory_map(mmap_buf).unwrap_success();
    // system_table.boot_services().memory_map(mmap_buf).unwrap_success();
    let mut root_dir = {
        let loaded_image = system_table
            .boot_services()
            .handle_protocol::<LoadedImage>(image)
            .unwrap_success()
            .get();
        let device = unsafe {(*loaded_image).device()};
        let file_system = system_table.boot_services().handle_protocol::<SimpleFileSystem>(device).unwrap_success().get();
        unsafe { (*file_system).open_volume().unwrap_success() }
    };
    let mut mmap_file = {
        use uefi::proto::media::file::{File, FileMode, FileAttribute};
        use uefi::proto::media::file::RegularFile;
        let mmap_file_handle = root_dir.open(
            "mmap_file.txt",
            FileMode::CreateReadWrite,
            FileAttribute::empty()
        ).unwrap_success();
        let file = match mmap_file_handle.into_type().expect_success("Failed to into_type") {
            FileType::Regular(file) => Some(file),
            _ => None,
        }.expect("Unexpected file type");
    
        FileWriter(file)
    };
    // save to file
    writeln!(mmap_file, "Image Base: {:x}", image_base);
    writeln!(mmap_file, "Idx, Type, Type(name), Start, NumOfPages, Attr").unwrap();
    for (i, desc) in desc_iter.enumerate() {
        writeln!(
            system_table.stdout(),
            "{}, {:x}, {:?}, {:x}, {}, {:x}",
            i, desc.ty.0, desc.ty, desc.phys_start, desc.page_count, desc.att
        ).unwrap();
        writeln!(
            mmap_file,
            "{}, {:x}, {:?}, {:x}, {}, {:x}",
            i, desc.ty.0, desc.ty, desc.phys_start, desc.page_count, desc.att
        ).unwrap();
    }
    mmap_file.0.flush().unwrap_success();
    // ------------------------------------------------------

    // frame buffer
    let frame_buffer = unsafe { get_frame_buffer(&system_table) };

    // read kernel file
    let mut root_dir = {
        use uefi::proto::loaded_image::LoadedImage;
        let loaded_image = system_table
            .boot_services()
            .handle_protocol::<LoadedImage>(image)
            .unwrap_success()
            .get();
        let device = unsafe { (*loaded_image).device() };
        let file_system = system_table
            .boot_services()
            .handle_protocol::<SimpleFileSystem>(device)
            .unwrap_success()
            .get();
        unsafe { (*file_system).open_volume().unwrap_success() }
    };
    let kernel_file = root_dir
        .open("potatOS.elf", FileMode::Read, FileAttribute::READ_ONLY)
        .unwrap_success();
    let mut kernel_file = unsafe { RegularFile::new(kernel_file) };

    let buf = &mut [0u8; 4000];
    let info: &mut FileInfo = kernel_file.get_info(buf).unwrap_success();
    let kernel_file_size = info.file_size() as usize;
    let kernel_file_buf: &mut [u8] = {
        let addr = system_table
            .boot_services()
            .allocate_pool(MemoryType::LOADER_DATA, kernel_file_size)
            .unwrap_success();
        unsafe { core::slice::from_raw_parts_mut(addr, kernel_file_size) }
    };

    kernel_file.read(kernel_file_buf).unwrap_success();
    kernel_file.close();

    // load kernel and retreive entry point
    use core::cmp;
    use goblin::elf;
    let kernel_elf = elf::Elf::parse(&kernel_file_buf).unwrap();
    let mut kernel_start = usize::MAX;
    let mut kernel_end = usize::MIN;
    // kernel start and size
    for pheader in kernel_elf
        .program_headers
        .iter()
        .filter(|ph| ph.p_type == elf::program_header::PT_LOAD)
    {
        kernel_start = cmp::min(kernel_start, pheader.p_vaddr as usize);
        kernel_end = cmp::max(kernel_end, (pheader.p_vaddr + pheader.p_memsz) as usize);
    }
    writeln!(
        system_table.stdout(),
        "Kernel: {:#x} - {:#x}",
        kernel_start,
        kernel_end
    )
    .unwrap();

    system_table
        .boot_services()
        .allocate_pages(
            uefi::table::boot::AllocateType::Address(kernel_start),
            MemoryType::LOADER_DATA,
            (kernel_end - kernel_start + 0xfff) / 0x1000,
        )
        .unwrap_success();

    // writeln!(system_table.stdout(), "{}", kernel_elf.program_headers.len()).unwrap();
    for pheader in kernel_elf
        .program_headers
        .iter()
        .filter(|item| item.p_type == elf::program_header::PT_LOAD)
    {
        let offset = pheader.p_offset as usize; // offset in file
        let file_size = pheader.p_filesz as usize; // LOAD segment file size
        let mem_size = pheader.p_memsz as usize; // LOAD segment memory size
        let mut load_dest =
            unsafe { slice::from_raw_parts_mut(pheader.p_vaddr as *mut u8, mem_size) };
        // maybe optimized out?
        load_dest[..file_size].copy_from_slice(&kernel_file_buf[offset..offset + file_size]);
        load_dest[file_size..].fill(0);
    }

    let entry_point = {
        let addr = kernel_elf.entry;
        unsafe { core::mem::transmute::<u64, EntryFn>(addr) }
    };

    writeln!(system_table.stdout(), "entry point: {:#x}", entry_point as u64).unwrap();

    // test entry (before exiting boot services)
    // writeln!(system_table.stdout(), "{:?}", frame_buffer).unwrap();
    // entry_point(frame_buffer);

    writeln!(system_table.stdout(), "exiting boot services").unwrap();
    // exit boot services (and retreive memory_map)
    uefi::alloc::exit_boot_services();
    let (_system_table, _memory_map) = system_table
        .exit_boot_services(image, mmap_storage)
        .unwrap_success();

    entry_point(frame_buffer); //

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    panic!("out of memory")
}
