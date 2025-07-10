#![no_std]
#![no_main]

extern crate nx;

use core::panic;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::ipc::sf;
use nx::service;
use nx::svc;
use nx::util;

use simple_service_server::{DemoService, IDemoServiceClient};

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    } else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[no_mangle]
pub fn main() {
    let mut demo_service_client = service::new_service_object::<DemoService>().unwrap();

    let demo = "demo";
    let mut omed = [0u8; 0x100];

    demo_service_client.sample_command(
        0x7,
        0x82,
        sf::InAutoSelectBuffer::from_array(demo.as_bytes()),
        sf::InOutAutoSelectBuffer::from_mut_array(&mut omed),
    ).unwrap();

}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
