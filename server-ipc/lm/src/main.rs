#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use nx::result::*;
use nx::util;
use nx::wait;
use nx::thread;
use nx::diag::abort;
use nx::ipc::sf;
use nx::ipc::server;
use nx::service;
use nx::service::psc;
use nx::service::psc::IPmModule;
use nx::service::psc::IPmService;
use nx::fs;
use core::panic;

rrt0_define_default_module_name!();

mod ipc;
mod logger;

const CUSTOM_HEAP_SIZE: usize = 0x8000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_SIZE] = [0; CUSTOM_HEAP_SIZE];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(CUSTOM_HEAP.as_mut_ptr(), CUSTOM_HEAP_SIZE)
    }
}

#[allow(unreachable_code)]
pub fn pm_module_main() -> Result<()> {
    let psc = service::new_service_object::<psc::PmService>()?;
    let module = psc.get().get_pm_module()?.to::<psc::PmModule>();

    let event_handle = module.get().initialize(psc::ModuleId::Lm, sf::Buffer::empty())?;
    loop {
        wait::wait_handles(&[event_handle.handle], -1)?;

        let (state, _flags) = module.get().get_request()?;
        match state {
            psc::State::FullAwake | psc::State::MinimumAwake | psc::State::EssentialServicesAwake => logger::set_log_enabled(true),
            _ => logger::set_log_enabled(false)
        };

        module.get().acknowledge_ex(state)?;
    }

    Ok(())
}

pub fn pm_module_thread(_: &()) {
    pm_module_main().unwrap();
}

const POINTER_BUF_SIZE: usize = 0x400;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

#[no_mangle]
pub fn main() -> Result<()> {
    thread::get_current_thread().name.set_str("lm.Main");
    fs::initialize_fspsrv_session()?;
    fs::mount_sd_card("sdmc")?;
    logger::initialize()?;

    let mut pm_module_thread = thread::Thread::new(pm_module_thread, &(), "lm.PmModule", 0x2000)?;
    pm_module_thread.initialize(38, -2)?;
    pm_module_thread.start()?;

    let mut manager = Manager::new()?;
    manager.register_service_server::<ipc::LogService>()?;
    manager.loop_process()?;

    pm_module_thread.join()?;
    fs::finalize_fspsrv_session();
    fs::unmount_all();
    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<logger::SelfLogger>(info, abort::AbortLevel::SvcBreak())
}