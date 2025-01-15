#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
use alloc::string::String;

extern crate nx;
use nx::fs;
use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;

use core::panic;

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    }
    else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    // Initializing this is not mandatory, but it's helpful for fs to automatically mount the SD by itself
    fs::initialize_fspsrv_session()?;
    fs::mount_sd_card("sdmc")?;

    let mut hbmenu_nro = fs::open_file("sdmc:/hbmenu.nro", fs::FileOpenOption::Read())?;
    hbmenu_nro.seek(fs::SeekFrom::Start(0x10))?; // Skip NRO start (https://switchbrew.org/wiki/NRO)
    let nro_magic: u32 = hbmenu_nro.read_val()?;

    let nro_magic_msg = format!("hbmenu NRO magic: {:#X}", nro_magic);
    let mut log_file = fs::open_file("sdmc:/fs-test-log.log", fs::FileOpenOption::Create() | fs::FileOpenOption::Write() | fs::FileOpenOption::Append())?;
    log_file.write_array(nro_magic_msg.as_bytes())?;

    fs::unmount_all();
    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}