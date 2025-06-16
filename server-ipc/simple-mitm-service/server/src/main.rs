#![no_std]
#![no_main]

extern crate alloc;

extern crate paste;

use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::ipc::server;
use nx::result::*;
use nx::service::psm::IPsmServer;
use nx::service::sm;
use nx::util;

use core::panic;
use core::ptr::addr_of_mut;

pub struct PsmMitmServer;

impl IPsmServer for PsmMitmServer {
    fn get_battery_charge_percentage(&mut self) -> Result<u32> {
        Ok(69)
    }
}

impl server::ISessionObject for PsmMitmServer {
    fn try_handle_request_by_id(
        &mut self,
        req_id: u32,
        protocol: nx::ipc::CommandProtocol,
        server_ctx: &mut server::ServerContext,
    ) -> Option<Result<()>> {
        <Self as IPsmServer>::try_handle_request_by_id(self, req_id, protocol, server_ctx)
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
#[allow(static_mut_refs)] // :(
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe { util::PointerAndSize::new(addr_of_mut!(CUSTOM_HEAP) as _, CUSTOM_HEAP.len()) }
}

const POINTER_BUF_SIZE: usize = 0;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

#[no_mangle]
pub fn main() {
    let mut manager = Manager::new().unwrap();
    manager.register_mitm_service_server::<PsmMitmServer>().unwrap();
    manager.loop_process().unwrap();
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
