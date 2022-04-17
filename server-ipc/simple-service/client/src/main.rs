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
use nx::ipc::sf;
use nx::ipc::client;
use nx::service;
use nx::service::sm;
use nx::version;

use core::panic;

// Same interface as /server project

ipc_sf_define_interface_trait! {
    trait IDemoService {
        sample_command [123, version::VersionInterval::all()]: (u32s_buf: sf::OutPointerBuffer<u32>) => ();
    }
}

pub struct DemoService {
    session: sf::Session
}

impl sf::IObject for DemoService {
    ipc_sf_object_impl_default_command_metadata!();
}

impl client::IClientObject for DemoService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl IDemoService for DemoService {
    fn sample_command(&mut self, u32s_buf: sf::OutPointerBuffer<u32>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 123] (u32s_buf) => ())
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

    let mut u32s: [u32; 5] = [0; 5];
    demo_srv.get().sample_command(sf::Buffer::from_mut_array(&mut u32s))?;

    diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "u32 list after sample_command: {} {} {} {} {}", u32s[0], u32s[1], u32s[2], u32s[3], u32s[4]);

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertLevel::FatalThrow())
}