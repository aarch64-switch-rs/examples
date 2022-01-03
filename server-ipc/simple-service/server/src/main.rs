#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

#[macro_use]
extern crate alloc;

extern crate paste;

use nx::result::*;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::ipc::sf;
use nx::ipc::server;

use core::panic;

pub trait IDemoService {
    ipc_cmif_interface_define_command!(test_buf: (buf: sf::OutPointerBuffer) => ());
}

pub struct DemoService {
    session: sf::Session
}

impl IDemoService for DemoService {
    fn test_buf(&mut self, buf: sf::OutPointerBuffer) -> Result<()> {
        diag_log!(log::LmLogger { log::LogSeverity::Trace, true } => "Buffer: {:p}, size: {}", buf.buf, buf.size);
        if (buf.size > 0) && !buf.buf.is_null() {
            let buf32 = buf.buf as *mut u32;
            let len = buf.size / core::mem::size_of::<u32>();
            for i in 0..len {
                diag_log!(log::LmLogger { log::LogSeverity::Trace, true } => "Setting {}...", i);
                unsafe { *buf32.offset(i as isize) = i as u32 };
            }
        }
        
        Ok(())
    }
}

impl sf::IObject for DemoService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(test_buf: 1)
        ]
    }
}

impl server::IServerObject for DemoService {
    fn new() -> Self {
        Self { session: sf::Session::new() }
    }
}

impl server::IService for DemoService {
    fn get_name() -> &'static str {
        nul!("dmo-srv")
    }

    fn get_max_sesssions() -> i32 {
        0x40
    }
}

// We're using 128KB of heap
static mut STACK_HEAP: [u8; 0x20000] = [0; 0x20000];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(STACK_HEAP.as_mut_ptr(), STACK_HEAP.len())
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    const POINTER_BUF_SIZE: usize = 0x400;
    let mut manager: server::ServerManager<POINTER_BUF_SIZE> = server::ServerManager::new()?;

    manager.register_service_server::<DemoService>()?;
    manager.loop_process()?;

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, assert::AssertLevel::FatalThrow())
}