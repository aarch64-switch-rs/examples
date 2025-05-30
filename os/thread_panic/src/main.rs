#![feature(panic_can_unwind)]
#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

use alloc::boxed::Box;
use nx::diag::abort;
use nx::diag::log::{lm::LmLogger, LogSeverity};
use nx::diag_log;
use nx::result::*;
use nx::thread;
use nx::util;

use core::any::Any;
use core::panic;

nx::rrt0_define_module_name!("thread-panic");
nx::rrt0_initialize_heap!();

#[no_mangle]
pub fn main() -> Result<()> {
    diag_log!(LmLogger { LogSeverity::Trace, false } => "Starting threads...\n");

    let thread_result = thread::Builder::new()
        .stack_size(0x8000 * 4)
        .spawn(move || {
            panic!("thread panic to catch in unwinding");
        })
        .expect("failed to spawn thread")
        .join();

    let result_string = format!("thread result: {:?}\n", thread_result);
    diag_log!(LmLogger { LogSeverity::Trace, false } => "{}", result_string);

    diag_log!(LmLogger { LogSeverity::Trace, false } => "Done!\n");

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    if info.can_unwind() {
        let message = format!("{:?}", info);
        unwinding::panic::begin_panic(Box::new(message) as Box<dyn Any + Send>);
    }
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
