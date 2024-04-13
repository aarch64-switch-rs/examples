#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use nx::result::*;
use nx::util;
use nx::diag::abort;
use nx::diag::log::{lm::LmLogger, LogSeverity};
use nx::ipc::sf;
use nx::ipc::server;
use nx::service::sm;
use nx::version;

use core::panic;

// Same interface as /client project

ipc_sf_define_interface_trait! {
    trait IDemoService {
        sample_command [999, version::VersionInterval::all()]: (a: u32, b: u64, c: sf::InAutoSelectBuffer<u8>, d: sf::OutAutoSelectBuffer<u8>) => ();
    }
}

pub struct DemoService {
    dummy_session: sf::Session
}

impl IDemoService for DemoService {
    fn sample_command(&mut self, a: u32, b: u64, c: sf::InAutoSelectBuffer<u8>, d: sf::OutAutoSelectBuffer<u8>) -> Result<()> {
        diag_log!(LmLogger { LogSeverity::Trace, true } => "a: {}", a);
        diag_log!(LmLogger { LogSeverity::Trace, true } => "b: {}", b);
        diag_log!(LmLogger { LogSeverity::Trace, true } => "c len: {}", c.get_string().len());
        diag_log!(LmLogger { LogSeverity::Trace, true } => "d len: {}", d.get_string().len());
        
        Ok(())
    }
}

impl sf::IObject for DemoService {
    ipc_sf_object_impl_default_command_metadata!();

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.dummy_session
    }
}

impl server::ISessionObject for DemoService {}

impl server::IServerObject for DemoService {
    fn new() -> Self {
        Self {
            dummy_session: sf::Session::new()
        }
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
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}