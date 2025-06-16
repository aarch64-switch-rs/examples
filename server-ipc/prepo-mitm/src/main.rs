#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use core::panic;
use core::ptr::addr_of_mut;
use nx::diag::abort;
use nx::diag::log::{lm::LmLogger, LogSeverity};
use nx::fs;
use nx::ipc::server;
use nx::util;
use nx::version;

mod prepo;

const CUSTOM_HEAP_SIZE: usize = 0x4000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_SIZE] = [0; CUSTOM_HEAP_SIZE];

#[no_mangle]
#[allow(static_mut_refs)] // :(
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    util::PointerAndSize::new(addr_of_mut!(CUSTOM_HEAP) as _, CUSTOM_HEAP_SIZE)
}

const POINTER_BUF_SIZE: usize = 0x1000;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

#[no_mangle]
pub fn main() {
    diag_log!(LmLogger { LogSeverity::Info, true } => "Hello there!\n");

    fs::initialize_fspsrv_session().expect("Error starting filesystem services");
    fs::mount_sd_card("sdmc").expect("Failed to mount sd card");

    let mut manager = Manager::new().unwrap();

    // Services present in all versions so far
    manager.register_mitm_service_server::<prepo::PrepoServiceMitmServer<{ prepo::SERVICE_TYPE_MANAGER }>>().unwrap();
    manager.register_mitm_service_server::<prepo::PrepoServiceMitmServer<{ prepo::SERVICE_TYPE_USER }>>().unwrap();

    // TODO: fix this mitm, keeps getting stuck on boot when am accesses it
    // manager.register_mitm_service_server::<prepo::PrepoService<{ prepo::SERVICE_TYPE_SYSTEM }>>().unwrap();

    if version::get_version() > version::Version::new(5, 1, 0) {
        // 6.0.0 -> (...) has "prepo:a2"
        manager.register_mitm_service_server::<prepo::PrepoServiceMitmServer<{ prepo::SERVICE_TYPE_ADMIN2 }>>().unwrap();
    } else {
        // 1.0.0 -> 5.1.0 has "prepo:a"
        manager.register_mitm_service_server::<prepo::PrepoServiceMitmServer<{ prepo::SERVICE_TYPE_ADMIN }>>().unwrap();
    }

    diag_log!(LmLogger { LogSeverity::Info, true } => "Looping...\n");
    manager.loop_process().unwrap();

    fs::unmount_all();
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::SvcBreak())
}
