#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
use alloc::string::String;

#[macro_use]
extern crate nx;
use nx::fs;
use nx::svc;
use nx::result::*;
use nx::results;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::ipc::sf;
use nx::service;
use nx::service::applet;
use nx::service::applet::IAllSystemAppletProxiesService;
use nx::service::applet::ILibraryAppletProxy;
use nx::service::applet::ILibraryAppletCreator;
use nx::service::applet::ILibraryAppletAccessor;
use nx::service::applet::IStorage;
use nx::service::applet::IStorageAccessor;

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
    fs::initialize()?;
    fs::mount_sd_card("sdmc")?;

    let mut hbmenu_nro = fs::open_file(String::from("sdmc:/hbmenu.nro"), fs::FileOpenOption::Read())?;
    let nro_magic: u32 = hbmenu_nro.read_val()?;

    let nro_magic_msg = format!("hbmenu NRO magic: {:#X}", nro_magic);
    let mut msg_log = fs::open_file(String::from("sdmc:/fs-test.log"), fs::FileOpenOption::Create() | fs::FileOpenOption::Write() | fs::FileOpenOption::Append())?;
    msg_log.write(nro_magic_msg.as_ptr(), nro_magic_msg.len())?;

    fs::finalize();
    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow)
}