#![no_std]
#![no_main]

extern crate alloc;

use uefi::{Status, Handle, table::{Boot, SystemTable, boot::{MemoryDescriptor, MemoryType}},
    prelude::entry, table::{cfg, boot::{OpenProtocolParams, OpenProtocolAttributes, SearchType}}, proto::console::gop::GraphicsOutput};
use uefi_services::{init, println};

use core::{mem, slice};

#[entry]
fn efi_main(
    handle: Handle,
    mut system_table: SystemTable<Boot>,
) -> Status {
    init(&mut system_table).unwrap();

    let mut config_entries = system_table.config_table().iter();
    let rsdp_addr = config_entries
        .find(|entry| matches!(entry.guid, cfg::ACPI_GUID | cfg::ACPI2_GUID))
        .map(|entry| entry.address)
        .unwrap();

    let gop_handle = {
        let gop_handles = system_table.boot_services()
            .locate_handle_buffer(SearchType::from_proto::<GraphicsOutput>())
            .unwrap();

        if gop_handles.is_empty() {
            panic!("Could not find any GraphicsOutput Handle");
        }

        gop_handles[0].clone()
    };

    let mut gop = unsafe { system_table.boot_services().open_protocol::<GraphicsOutput>(
        OpenProtocolParams {
            handle: gop_handle,
            agent: handle,
            controller: None
        },
        OpenProtocolAttributes::GetProtocol)
        .unwrap()
    };

    println!("Hello, UEFI!");
    println!("rsdp addr: {:?}", rsdp_addr);
    println!("current gop mode: {:?}", gop.current_mode_info());
    println!("framebuffer at: {:#p}", gop.frame_buffer().as_mut_ptr());

    drop(gop);

    let mmap_storage = {
        let max_mmap_size = system_table.boot_services().memory_map_size().map_size
            + 8 * mem::size_of::<MemoryDescriptor>();
        let ptr = system_table
            .boot_services()
            .allocate_pool(MemoryType::LOADER_DATA, max_mmap_size).unwrap();
        unsafe { slice::from_raw_parts_mut(ptr, max_mmap_size) }
    };

    uefi::global_allocator::exit_boot_services();
    let (system_table, memory_map) = system_table.exit_boot_services();

    static KERNEL: &[u8] = include_bytes!("../../target/x86_64-unknown-ampere/debug/ampere-kernel");

    loop {}
}