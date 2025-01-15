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

use simple_service_server::IDemoServiceServer;

use core::panic;
use core::ptr::addr_of_mut;

pub struct DemoServiceServer;

impl IDemoServiceServer for DemoServiceServer {
    fn sample_command(&mut self, a: u32, b: u64, c: sf::InAutoSelectBuffer<u8>, d: sf::OutAutoSelectBuffer<u8>) -> Result<()> {
        diag_log!(LmLogger { LogSeverity::Trace, true } => "a: {}", a);
        diag_log!(LmLogger { LogSeverity::Trace, true } => "b: {}", b);
        diag_log!(LmLogger { LogSeverity::Trace, true } => "c len: {}", c.get_string().len());
        diag_log!(LmLogger { LogSeverity::Trace, true } => "d len: {}", d.get_string().len());
        
        Ok(())
    }
}

impl server::ISessionObject for DemoServiceServer {
    fn try_handle_request_by_id(&mut self, req_id: u32, protocol: nx::ipc::CommandProtocol, server_ctx: &mut server::ServerContext) -> Option<Result<()>> {
        <Self as IDemoServiceServer>::try_handle_request_by_id(self, req_id, protocol, server_ctx)
    }
}

impl server::IServerObject for DemoServiceServer {
    fn new() -> Self {
        Self
    }
}

impl server::IService for DemoServiceServer {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("dmo-srv")
    }
    fn get_max_sesssions() -> i32 {
        20
    }
}

// We're using 128KB of heap
const CUSTOM_HEAP_LEN: usize = 0x20000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_LEN] = [0; CUSTOM_HEAP_LEN];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(addr_of_mut!(CUSTOM_HEAP) as _, CUSTOM_HEAP_LEN)
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    const POINTER_BUF_SIZE: usize = 0x400;
    let mut manager: server::ServerManager<POINTER_BUF_SIZE> = server::ServerManager::new()?;

    manager.register_service_server::<DemoServiceServer>()?;
    manager.loop_process()?;

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}