#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

use nx::svc;
use nx::result::*;
use nx::results;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::service;
use nx::ipc::sf;

use core::panic;

// Same interface as /server project
pub trait IAccountServiceForApplication {
    ipc_interface_define_command!(get_user_count: () => (out_value: u32));
}

pub struct AccountServiceForApplication {
    session: sf::Session
}

impl sf::IObject for AccountServiceForApplication {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
    
    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_interface_make_command_meta!(get_user_count: 0)
        ]
    }
}

impl IAccountServiceForApplication for AccountServiceForApplication {
    fn get_user_count(&mut self) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (count: u32))
    }
}

impl service::IClientObject for AccountServiceForApplication {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl service::IService for AccountServiceForApplication {
    fn get_name() -> &'static str {
        nul!("acc:u0")
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
    let acc = service::new_service_object::<AccountServiceForApplication>()?;

    let count = acc.get().get_user_count()?;
    diag_log!(log::LmLogger { log::LogSeverity::Error, true } => "Got user count: {}", count);

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow)
}