#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::abort;
use nx::diag::log::{lm::LmLogger, LogSeverity};
use nx::ipc::sf;
use nx::ipc::client;
use nx::service;
use nx::service::sm;
use nx::version;
use alloc::vec::Vec;
use core::panic;

// Same interface as /server project

ipc_sf_define_interface_trait! {
    trait IDemoService {
        sample_command [999, version::VersionInterval::all()]: (a: u32, b: u64, c: sf::InAutoSelectBuffer<u8>, d: sf::OutAutoSelectBuffer<u8>) => ();
    }
}

pub struct DemoService {
    session: sf::Session
}

impl sf::IObject for DemoService {
    ipc_sf_object_impl_default_command_metadata!();

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl client::IClientObject for DemoService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IDemoService for DemoService {
    fn sample_command(&mut self, a: u32, b: u64, c: sf::InAutoSelectBuffer<u8>, d: sf::OutAutoSelectBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 999] (a, b, c, d) => ())
    }
}

impl service::IService for DemoService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("dmo-srv")
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
    let demo_srv = service::new_service_object::<DemoService>()?;

    let demo = "demo";
    let mut mode: Vec<u8> = vec![0u8; 0x100];

    demo_srv.get().sample_command(0x7, 0x82, sf::InAutoSelectBuffer::from_array(demo.as_bytes()), sf::OutAutoSelectBuffer::from_mut_array(&mut mode))?;

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}