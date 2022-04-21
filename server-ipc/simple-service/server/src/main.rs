#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use nx::result::*;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::ipc::sf;
use nx::ipc::server;
use nx::service::sm;
use nx::version;

use core::panic;

// Same interface as /client project

ipc_sf_define_interface_trait! {
    trait IDemoService {
        sample_command [123, version::VersionInterval::all()]: (u32s_buf: sf::OutPointerBuffer<u32>) => ();
    }
}

pub struct DemoService {}

impl IDemoService for DemoService {
    fn sample_command(&mut self, u32s_buf: sf::OutPointerBuffer<u32>) -> Result<()> {
        diag_log!(log::LmLogger { log::LogSeverity::Trace, true } => "List count: {}", u32s_buf.get_count());

        let u32s = u32s_buf.get_mut_slice();
        for u32_val in u32s {
            // For each u32 we got sent, replace it as <orig-val> * 3
            let orig_val = *u32_val;
            *u32_val = orig_val * 3;
            diag_log!(log::LmLogger { log::LogSeverity::Trace, true } => "Updating {} -> {}...", orig_val, *u32_val);
        }
        
        Ok(())
    }
}

impl sf::IObject for DemoService {
    ipc_sf_object_impl_default_command_metadata!();
}

impl server::ISessionObject for DemoService {}

impl server::IServerObject for DemoService {
    fn new() -> Self {
        Self {}
    }
}

impl server::IService for DemoService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("dmo-srv")
    }

    fn get_max_sesssions() -> i32 {
        0x40
    }
}

// We're using 128KB of heap
const CUSTOM_HEAP_LEN: usize = 0x20000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_LEN] = [0; CUSTOM_HEAP_LEN];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(CUSTOM_HEAP.as_mut_ptr(), CUSTOM_HEAP.len())
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    const POINTER_BUF_SIZE: usize = 0x400;
    let mut manager: server::ServerManager<POINTER_BUF_SIZE> = server::ServerManager::new()?;

    manager.register_service_server::<DemoService>()?;
    manager.loop_process()?;

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertLevel::FatalThrow())
}