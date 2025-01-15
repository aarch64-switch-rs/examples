#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

use nx::diag_log;
use nx::thread;
use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::abort;
use nx::diag::log::{lm::LmLogger, LogSeverity};

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
    diag_log!(LmLogger { LogSeverity::Trace, false } => "Starting threads...\n");

    let t1_args = (1, 2, 3, 'c', 'd', 'b', 'a');
    let t1 = thread::spawn(move || {
        let (i1, i2, i3, c1, c2, c3, c4) = t1_args;
        diag_log!(LmLogger { LogSeverity::Trace, false } => "Thread 1 sample params: {} {} {} {} {} {} {}\n", i1, i2, i3, c1, c2, c3, c4);
        for i in 0..5 {
            diag_log!(LmLogger { LogSeverity::Trace, false } => "Hi from thread 1 (ID: {}) {}\n", nx::svc::get_thread_id(nx::svc::CURRENT_THREAD_PSEUDO_HANDLE).unwrap_or(u64::MAX), i);
        }
    });

    let t2 = thread::spawn(move || {
        for i in 0..5 {
            diag_log!(LmLogger { LogSeverity::Trace, false } => "Ohayo from thread 2 (ID: {}) {}\n", nx::svc::get_thread_id(nx::svc::CURRENT_THREAD_PSEUDO_HANDLE).unwrap_or(u64::MAX), i);
        }
    });

    let t3 = thread::spawn(move || {
        for i in 0..5 {
            diag_log!(LmLogger { LogSeverity::Trace, false } => "Sup from thread 3 (ID: {}) {}\n", nx::svc::get_thread_id(nx::svc::CURRENT_THREAD_PSEUDO_HANDLE).unwrap_or(u64::MAX), i);
        }
    });


    t1.join();
    t2.join();
    t3.join();

    diag_log!(LmLogger { LogSeverity::Trace, false } => "Done!\n");

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}