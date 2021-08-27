#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate nx;
use nx::thread;
use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::assert;
use nx::diag::log;

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
    diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Starting threads...\n");

    let mut t1 = thread::Thread::new(move |&(i1, i2, i3, c1, c2, c3, c4)| {
        diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Thread 1 sample params: {} {} {} {} {} {} {}\n", i1, i2, i3, c1, c2, c3, c4);
        for i in 0..5 {
            diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Hi from thread 1 (ID: {}) {}\n", thread::get_current_thread().get_id().unwrap(), i);
        }
    }, &(1, 2, 3, 'c', 'd', 'b', 'a'), "Thread1", 0x1000)?;
    t1.initialize(thread::PRIORITY_AUTO, -2)?;

    let mut t2 = thread::Thread::new(move |()| {
        for i in 0..5 {
            diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Ohayo from thread 2 (ID: {}) {}\n", thread::get_current_thread().get_id().unwrap(), i);
        }
    }, &(), "Thread2", 0x1000)?;
    t2.initialize(thread::PRIORITY_AUTO, -2)?;

    let mut t3 = thread::Thread::new(move |()| {
        for i in 0..5 {
            diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Sup from thread 3 (ID: {}) {}\n", thread::get_current_thread().get_id().unwrap(), i);
        }
    }, &(), "Thread3", 0x1000)?;
    t3.initialize(thread::PRIORITY_AUTO, -2)?;

    diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Thread IDs: {} {} {}\n", t1.get_id()?, t2.get_id()?, t3.get_id()?);

    t1.start()?;
    t2.start()?;
    t3.start()?;

    t1.join()?;
    t2.join()?;
    t3.join()?;

    diag_log!(log::LmLogger { log::LogSeverity::Trace, false } => "Done!\n");

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow)
}