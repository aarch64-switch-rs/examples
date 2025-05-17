#![no_std]
#![no_main]

#[macro_use]
extern crate nx;

extern crate alloc;
extern crate paste;

use alloc::string::ToString;
use nx::result::*;
use nx::util;
use nx::thread;
use nx::diag::abort;
use nx::diag::log;
use nx::ipc::server;
use nx::ipc::sf::ncm;
use nx::fs;
use core::panic;
use nx::ipc::sf::ldr::IShellInterface;

rrt0_define_default_module_name!();

const CUSTOM_HEAP_SIZE: usize = 0x10000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_SIZE] = [0; CUSTOM_HEAP_SIZE];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(CUSTOM_HEAP.as_mut_ptr(), CUSTOM_HEAP_SIZE)
    }
}

const POINTER_BUF_SIZE: usize = 0x1000;
type Manager = server::ServerManager<POINTER_BUF_SIZE>;

// Example game to take over: Animal Crossing New Horizons
const TAKE_OVER_APP_ID: ncm::ProgramId = ncm::ProgramId(0x01006F8002326000);

#[no_mangle]
pub fn main() -> Result<()> {
    thread::get_current_thread().name.set_str("ecs.Main");
    fs::initialize_fspsrv_session()?;
    fs::mount_sd_card("sdmc")?;

    let ldr_shel = nx::service::new_service_object::<nx::service::ldr::ShellInterface>()?;

    let handle = ldr_shel.get().atmosphere_register_external_code(TAKE_OVER_APP_ID)?;

    let subdir_ipc_fs = ::alloc::boxed::Box::new(nx::fs::subdir::FileSystem::new("sdmc:/dummy".to_string()));

    let mut manager = Manager::new()?;
    manager.register_session(handle.handle, subdir_ipc_fs);
    manager.loop_process()?;

    fs::finalize_fspsrv_session();
    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::lm::LmLogger>(info, abort::AbortLevel::SvcBreak())
}