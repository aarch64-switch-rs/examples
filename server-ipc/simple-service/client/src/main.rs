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
use nx::diag::log::lm::LmLogger;
use nx::ipc::sf;
use nx::service;
use alloc::vec::Vec;
use core::panic;

use simple_service_server::{DemoService, IDemoService}; 

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
    let mut demo_srv = service::new_service_object::<DemoService>()?;

    let demo = "demo";
    let mut mode: Vec<u8> = vec![0u8; 0x100];

    demo_srv.sample_command(0x7, 0x82, sf::InAutoSelectBuffer::from_array(demo.as_bytes()), sf::OutAutoSelectBuffer::from_mut_array(mode.as_mut_slice()))?;

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}