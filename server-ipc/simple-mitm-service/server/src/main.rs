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
use nx::service::sm;
use nx::ipc::sf;
use nx::ipc::server;

use core::panic;

pub trait IPsmServer {
    ipc_cmif_interface_define_command!(get_battery_charge_percentage: () => (out_value: u32));
}

pub struct PsmServer {
    session: sf::Session
}

impl sf::IObject for PsmServer {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
    
    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(get_battery_charge_percentage: 0)
        ]
    }
}

impl IPsmServer for PsmServer {
    fn get_battery_charge_percentage(&mut self) -> Result<u32> {
        let stub: u32 = 69;
        diag_log!(log::LmLogger { log::LogSeverity::Trace, true } => "Returning stubbed battery percentage as {}%...\n", stub);
        Ok(stub)
    }
}

impl server::IMitmServerObject for PsmServer {
    fn new(_info: sm::MitmProcessInfo) -> Self {
        Self { session: sf::Session::new() }
    }
}

impl server::IMitmService for PsmServer {
    fn get_name() -> &'static str {
        nul!("psm")
    }

    fn should_mitm(_info: sm::MitmProcessInfo) -> bool {
        true
    }
}

pub const STACK_HEAP_SIZE: usize = 0x4000;
static mut STACK_HEAP: [u8; STACK_HEAP_SIZE] = [0; STACK_HEAP_SIZE];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(STACK_HEAP.as_mut_ptr(), STACK_HEAP.len())
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
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow)
}