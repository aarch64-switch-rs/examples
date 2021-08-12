#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::service;
use nx::ipc::sf;

use core::panic;

// Same interface as /server project
pub trait IPsmServer {
    ipc_cmif_interface_define_command!(get_battery_charge_percentage: () => (out_percentage: u32));
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
        ipc_client_send_request_command!([self.session.object_info; 0] () => (out_percentage: u32))
    }
}

impl service::IClientObject for PsmServer {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl service::IService for PsmServer {
    fn get_name() -> &'static str {
        nul!("psm")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    }
    else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    let psm = service::new_service_object::<PsmServer>()?;

    let battery_p = psm.get().get_battery_charge_percentage()?;
    diag_log!(log::LmLogger { log::LogSeverity::Trace, true } => "Battery percentage value: {}%\n", battery_p);

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow)
}