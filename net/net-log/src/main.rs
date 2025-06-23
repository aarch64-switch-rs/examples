#![no_std]
#![no_main]

extern crate alloc;

use core::net::Ipv4Addr;
use core::panic;

use nx::result::Result;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;

use nx::socket::net::UdpSocket;
use nx::sync::Mutex;
use nx::{svc, util};

nx::rrt0_define_module_name!("net-log");

#[unsafe(no_mangle)]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    } else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

static LOG_HOST: Ipv4Addr = Ipv4Addr::new(10,0,0,65);
static LOG_PORT: u16 = 5001;

pub struct LogImpl;

impl log::Log for LogImpl {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        LOG_STATIC.lock().is_some() && metadata.level() < log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        let mut handle = LOG_STATIC.lock();
        if let Some(writer) = handle.as_mut() {
            let _ = core::fmt::write(writer, format_args!("{} - {}", record.level(), record.args()));
        }
        
    }
    fn flush(&self) {
    }
}

static LOG_STATIC: Mutex<Option<UdpSocket>> = Mutex::new(None);

fn init_logger() -> Result<()> {
    let mut log_handle = LOG_STATIC.lock();
    if log_handle.is_some() {return Ok(());}

    nx::socket::initialize(nx::socket::BsdSrvkind::User, Default::default(), None)?;

    *log_handle = Some(nx::socket::net::UdpSocket::connect((LOG_HOST, LOG_PORT))?);

    Ok(())
}

#[unsafe(no_mangle)]
fn main() {
    init_logger().unwrap();

    let _ = log::set_logger(&LogImpl);

    log::set_max_level(log::LevelFilter::Debug);

    log::error!("error message");
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
