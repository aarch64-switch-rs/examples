#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

use nx::service::psm::PsmServer;
use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::abort;
use nx::diag::log::{lm::LmLogger, LogSeverity};
use nx::service;
use nx::ipc::sf;
use nx::ipc::client;
use nx::service::sm;
use nx::version;
use nx::service::psm::IPsmServer;

use core::panic;

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

    let battery_p = psm.get_battery_charge_percentage()?;
    diag_log!(LmLogger { LogSeverity::Trace, true } => "Battery percentage value: {}%\n", battery_p);

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}