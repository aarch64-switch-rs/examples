#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use nx::result::*;
use nx::util;
use nx::fs;
use nx::thread;
use nx::diag::assert;
use nx::diag::log;
use nx::ipc::server;
use nx::version;
use core::panic;

mod prepo;

const CUSTOM_HEAP_SIZE: usize = 0x4000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_SIZE] = [0; CUSTOM_HEAP_SIZE];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(CUSTOM_HEAP.as_mut_ptr(), CUSTOM_HEAP.len())
    }
}

const POINTER_BUF_SIZE: usize = 0x1000;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

#[no_mangle]
pub fn main() -> Result<()> {
    thread::get_current_thread().name.set_str("prepo-mitm.Main");
    diag_log!(log::LmLogger { log::LogSeverity::Info, true } => "Hello there!\n");

    fs::initialize_fspsrv_session()?;
    fs::mount_sd_card("sdmc")?;

    let mut manager = Manager::new()?;

    // Services present in all versions so far
    manager.register_mitm_service_server::<prepo::PrepoService<{ prepo::SERVICE_TYPE_MANAGER }>>()?;
    manager.register_mitm_service_server::<prepo::PrepoService<{ prepo::SERVICE_TYPE_USER }>>()?;

    // TODO: fix this mitm, keeps getting stuck on boot when am accesses it
    // manager.register_mitm_service_server::<prepo::PrepoService<{ prepo::SERVICE_TYPE_SYSTEM }>>()?;

    if version::get_version() > version::Version::new(5, 1, 0) {
        // 6.0.0 -> (...) has "prepo:a2"
        manager.register_mitm_service_server::<prepo::PrepoService<{ prepo::SERVICE_TYPE_ADMIN2 }>>()?;
    }
    else {
        // 1.0.0 -> 5.1.0 has "prepo:a"
        manager.register_mitm_service_server::<prepo::PrepoService<{ prepo::SERVICE_TYPE_ADMIN }>>()?;
    }

    diag_log!(log::LmLogger { log::LogSeverity::Info, true } => "Looping...\n");
    manager.loop_process()?;

    fs::finalize_fspsrv_session();
    fs::unmount_all();
    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertLevel::SvcBreak())
}