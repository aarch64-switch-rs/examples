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
use nx::service::psm::IPsmServerServer;

use core::panic;
use core::ptr::addr_of_mut;

pub struct PsmMitmServer;

impl IPsmServerServer for PsmMitmServer {
    fn get_battery_charge_percentage(&mut self) -> Result<u32> {
        Ok(69)
    }
}

impl server::ISessionObject for PsmMitmServer {
    fn try_handle_request_by_id(&mut self, req_id: u32, protocol: nx::ipc::CommandProtocol, server_ctx: &mut server::ServerContext) -> Option<Result<()>> {
        <Self as IPsmServerServer>::try_handle_request_by_id(self, req_id, protocol, server_ctx)
    }
}

impl server::IMitmServerObject for PsmMitmServer {
    fn new(_info: sm::mitm::MitmProcessInfo) -> Self {
        Self
    }
}

impl server::IMitmService for PsmMitmServer {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("psm")
    }

    fn should_mitm(_info: sm::mitm::MitmProcessInfo) -> bool {
        true
    }
}

pub const CUSTOM_HEAP_SIZE: usize = 0x40000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_SIZE] = [0; CUSTOM_HEAP_SIZE];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(addr_of_mut!(CUSTOM_HEAP) as _, CUSTOM_HEAP.len())
    }
}

const POINTER_BUF_SIZE: usize = 0;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

#[no_mangle]
pub fn main() -> Result<()> {
    let mut manager = Manager::new()?;
    manager.register_mitm_service_server::<PsmMitmServer>()?;
    manager.loop_process()?;

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}