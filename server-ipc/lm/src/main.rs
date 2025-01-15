#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use nx::result::*;
use nx::service::psc::IPmModule;
use nx::util;
use nx::wait;
use nx::thread;
use nx::diag::abort;
use nx::ipc::sf;
use nx::ipc::server;
use nx::service;
use nx::service::psc;
use nx::service::psc::IPmService;
use nx::fs;
use core::panic;
use core::ptr::addr_of_mut;

rrt0_define_default_module_name!();

mod ipc;
mod logger;

const CUSTOM_HEAP_SIZE: usize = 0x8000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_SIZE] = [0; CUSTOM_HEAP_SIZE];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(addr_of_mut!(CUSTOM_HEAP) as _, CUSTOM_HEAP_SIZE)
    }
}

pub fn pm_module_main() -> Result<()> {
    let psc = service::new_service_object::<psc::PmService>()?;
    let module = psc.get_pm_module()?;

    let event_handle = module.initialize(psc::ModuleId::Lm, sf::Buffer::empty())?;
    loop {
        wait::wait_handles(&[event_handle.handle], -1)?;

        let (state, _flags) = module.get_request()?;
        match state {
            psc::State::FullAwake | psc::State::MinimumAwake | psc::State::EssentialServicesAwake => logger::set_log_enabled(true),
            _ => logger::set_log_enabled(false)
        };

        module.acknowledge_ex(state)?;
    }
}

pub fn pm_module_thread(_: &()) {
    pm_module_main().unwrap();
}

const POINTER_BUF_SIZE: usize = 0x400;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

#[no_mangle]
pub fn main() -> Result<()> {
    thread::set_current_thread_name("lm.Main");
    fs::initialize_fspsrv_session()?;
    fs::mount_sd_card("sdmc")?;
    logger::initialize()?;

    let pm_module_thread = thread::Builder::new().name("lm.PmModule").stack_size(0x2000).spawn(|| {pm_module_main()})?;

    let mut manager = Manager::new()?;
    manager.register_service_server::<ipc::LogService>()?;
    manager.loop_process()?;

    pm_module_thread.join().expect("PmModule thread panicked.")?;

    fs::unmount_all();
    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<logger::SelfLogger>(info, abort::AbortLevel::SvcBreak())
}