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
use nx::ipc::sf;
use nx::service;

use core::panic;

pub trait IDemoService {
    ipc_interface_define_command!(test_buf: (buf: sf::OutPointerBuffer) => ());
}

pub struct DemoService {
    session: sf::Session
}

impl sf::IObject for DemoService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_interface_make_command_meta!(test_buf: 1)
        ]
    }
}

impl service::IClientObject for DemoService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IDemoService for DemoService {
    fn test_buf(&mut self, buf: sf::OutPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (buf) => ())
    }
}

impl service::IService for DemoService {
    fn get_name() -> &'static str {
        nul!("dmo-srv")
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
    let demosrv = service::new_service_object::<DemoService>()?;

    let mut data: [u32; 5] = [0; 5];
    demosrv.get().test_buf(sf::Buffer::from_mut(data.as_mut_ptr(), data.len() * core::mem::size_of::<u32>()))?;

    diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Got: {} {} {} {} {}", data[0], data[1], data[2], data[3], data[4]);

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}