#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate nx;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::diag::log::LogSeverity;
use nx::fs;
use nx::svc;
use nx::util;

use core::panic;

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    } else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[no_mangle]
pub fn main() {
    // Initializing this is not mandatory, but it's helpful for fs to automatically mount the SD by itself
    fs::initialize_fspsrv_session().expect("Error starting filesystem services");
    fs::mount_sd_card("sdmc").expect("Failed to mount sd card");

    let mut dir = fs::open_directory(
        "sdmc:/",
        fs::DirectoryOpenMode::ReadDirectories() | fs::DirectoryOpenMode::ReadFiles(),
    ).expect("Failed to open directory.");
    loop {
        if let Ok(Some(dd)) = dir.read_next() {
            diag_log!(LmLogger { LogSeverity::Trace, false } => "- {:?} ({:?})\n", dd.name, dd.entry_type);
        } else {
            break;
        }
    }

    fs::unmount_all();
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
