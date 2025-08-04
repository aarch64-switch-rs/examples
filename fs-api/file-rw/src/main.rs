#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::fs;
use nx::svc;
use nx::util;
use nx::fs::Write;

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

    let mut hbmenu_nro = fs::open_file("sdmc:/hbmenu.nro", fs::FileOpenOption::Read()).expect("Failed to load hbmenu executable");
    hbmenu_nro.seek(fs::SeekFrom::Start(0x10)).expect("File seek call failed"); // Skip NRO start (https://switchbrew.org/wiki/NRO)
    let nro_magic: u32 = hbmenu_nro.read_val().expect("Failed to read NRO magic number");

    let nro_magic_msg = format!("hbmenu NRO magic: {:#X}", nro_magic);
    let mut log_file = fs::open_file(
        "sdmc:/fs-test-log.log",
        fs::FileOpenOption::Create() | fs::FileOpenOption::Write() | fs::FileOpenOption::Append(),
    ).expect("Failed to open log file.");
    log_file.write_all(nro_magic_msg.as_bytes()).expect("Log file write failed.");

    fs::unmount_all();
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
