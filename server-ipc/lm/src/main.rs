#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use core::panic;
use nx::diag::abort;
use nx::fs;
use nx::ipc::server;
use nx::ipc::sf;
use nx::result::*;
use nx::service;
use nx::service::psc;
use nx::service::psc::IPmModuleClient;
use nx::service::psc::IPmClient;
use nx::thread;
use nx::util;
use nx::wait;

rrt0_define_default_module_name!();

mod ipc;
mod logger;

const CUSTOM_HEAP_SIZE: usize = 0x8000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_SIZE] = [0; CUSTOM_HEAP_SIZE];

#[no_mangle]
#[allow(static_mut_refs)] // :(
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    util::PointerAndSize::new(&raw mut CUSTOM_HEAP as _, CUSTOM_HEAP_SIZE)
}

pub fn pm_module_main() -> Result<()> {
    let psc = service::new_service_object::<psc::PmService>().unwrap();
    let module = psc.get_pm_module().unwrap();

    let event_handle = module.initialize(psc::ModuleId::Lm, sf::Buffer::from_array(&[])).unwrap();
    loop {
        wait::wait_handles(&[event_handle.handle], -1).unwrap();

        let (state, _flags) = module.get_request().unwrap();
        match state {
            psc::State::FullAwake
            | psc::State::MinimumAwake
            | psc::State::EssentialServicesAwake => logger::G_ENABLED.store(true, core::sync::atomic::Ordering::Relaxed),
            _ => logger::G_ENABLED.store(false, core::sync::atomic::Ordering::Relaxed),
        };

        module.acknowledge_ex(state).unwrap();
    }
}

pub fn pm_module_thread(_: &()) {
    pm_module_main().unwrap();
}

const POINTER_BUF_SIZE: usize = 0x400;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

#[no_mangle]
pub fn main() {
    thread::set_current_thread_name("lm.Main");
    fs::initialize_fspsrv_session().expect("Error starting filesystem services");
    fs::mount_sd_card("sdmc").expect("Failed to mount sd card");
    logger::initialize().unwrap();

    let pm_module_thread = thread::Builder::new()
        .name("lm.PmModule")
        .stack_size(0x2000)
        .spawn(|| pm_module_main()).unwrap();

    let mut manager = Manager::new().unwrap();
    manager.register_service_server::<ipc::LogService>().unwrap();
    manager.loop_process().unwrap();

    pm_module_thread
        .join()
        .expect("PmModule thread panicked.")
        .expect("PmModule returned an error.");

    fs::unmount_all();
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<logger::SelfLogger>(info, abort::AbortLevel::SvcBreak())
}
