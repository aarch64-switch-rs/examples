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
use nx::service::sm;
use nx::ipc::sf;
use nx::ipc::server;
use nx::version;

use core::panic;

// Same interface as /client project

ipc_sf_define_interface_trait! {
    trait IPsmServer {
        get_battery_charge_percentage [0, version::VersionInterval::all()]: () => (percentage: u32);
    }
}

pub struct PsmServer {
    dummy_session: sf::Session
}

impl sf::IObject for PsmServer {
    ipc_sf_object_impl_default_command_metadata!();

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.dummy_session
    }
}

impl IPsmServer for PsmServer {
    fn get_battery_charge_percentage(&mut self) -> Result<u32> {
        let stub: u32 = 69;
        diag_log!(LmLogger { LogSeverity::Trace, true } => "Returning fake/stubbed battery percentage as {}%...\n", stub);
        Ok(stub)
    }
}

impl server::ISessionObject for PsmServer {}

impl server::IMitmServerObject for PsmServer {
    fn new(_info: sm::mitm::MitmProcessInfo) -> Self {
        Self {
            dummy_session: sf::Session::new()
        }
    }
}

impl server::IMitmService for PsmServer {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("psm")
    }

    fn should_mitm(_info: sm::mitm::MitmProcessInfo) -> bool {
        true
    }
}

pub const CUSTOM_HEAP_SIZE: usize = 0x4000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_SIZE] = [0; CUSTOM_HEAP_SIZE];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(CUSTOM_HEAP.as_mut_ptr(), CUSTOM_HEAP.len())
    }
}

const POINTER_BUF_SIZE: usize = 0;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

#[no_mangle]
pub fn main() -> Result<()> {
    let mut manager = Manager::new()?;
    manager.register_mitm_service_server::<PsmServer>()?;
    manager.loop_process()?;

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}