#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate nx;
use nx::svc;
use nx::arm;
use nx::result::*;
use nx::results;
use nx::util;
use nx::diag::assert;
use nx::diag::log;
use nx::diag::log::Logger;
use nx::ipc::sf;
use nx::service;
use nx::service::applet;
use nx::service::applet::IAllSystemAppletProxiesService;
use nx::service::applet::ILibraryAppletProxy;
use nx::service::applet::ILibraryAppletCreator;
use nx::service::applet::ILibraryAppletAccessor;
use nx::service::applet::IStorage;
use nx::service::applet::IStorageAccessor;

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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommonArguments {
    pub version: u32,
    pub size: u32,
    pub la_api_version: u32,
    pub theme_color: u32,
    pub play_startup_sound: bool,
    pub pad: [u8; 7],
    pub system_tick: u64
}

impl CommonArguments {
    pub fn new(version: u32, la_api_version: u32, theme_color: u32, play_startup_sound: bool) -> Self {
        Self { version: version, size: core::mem::size_of::<Self>() as u32, la_api_version: la_api_version, theme_color: theme_color, play_startup_sound: play_startup_sound, pad: [0; 7], system_tick: arm::get_system_tick() }
    }
}

pub fn applet_test() -> Result<()> {
    let applet_proxy_srv = service::new_service_object::<applet::AllSystemAppletProxiesService>()?;
    
    let attr: applet::AppletAttribute = unsafe { core::mem::zeroed() };
    let lib_applet_proxy = applet_proxy_srv.get().open_library_applet_proxy(sf::ProcessId::new(), sf::Handle::from(svc::CURRENT_PROCESS_PSEUDO_HANDLE), sf::Buffer::from_var(&attr))?.to::<applet::LibraryAppletProxy>();
    let lib_applet_creator = lib_applet_proxy.get().get_library_applet_creator()?.to::<applet::LibraryAppletCreator>();
    let lib_applet_accessor = lib_applet_creator.get().create_library_applet(applet::AppletId::PlayerSelect, applet::LibraryAppletMode::AllForeground)?.to::<applet::LibraryAppletAccessor>();

    {
        let common_args = CommonArguments::new(1, 0x20000, 0, false);
        let storage = lib_applet_creator.get().create_storage(common_args.size as usize)?.to::<applet::Storage>();
        {
            let storage_accessor = storage.get().open()?.to::<applet::StorageAccessor>();
            storage_accessor.get().write(0, sf::Buffer::from_var(&common_args))?;
        }
        lib_applet_accessor.get().push_in_data(storage)?;
    }

    {
        let mut data: [u8; 0xA0] = [0; 0xA0];
        data[0x96] = 1;
        let storage = lib_applet_creator.get().create_storage(data.len())?.to::<applet::Storage>();
        {
            let storage_accessor = storage.get().open()?.to::<applet::StorageAccessor>();
            storage_accessor.get().write(0, sf::Buffer::from_const(data.as_ptr(), data.len()))?;
        }
        lib_applet_accessor.get().push_in_data(storage)?;
    }

    let event_handle = lib_applet_accessor.get().get_applet_state_changed_event()?;
    lib_applet_accessor.get().start()?;
    svc::wait_synchronization(&event_handle.handle, 1, -1)?;

    svc::close_handle(event_handle.handle)?;

    Ok(())
}

#[no_mangle]
pub fn main() -> Result<()> {
    if let Err(rc) = applet_test() {
        diag_result_log_assert!(log::LmLogger, assert::AssertMode::FatalThrow => rc);
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::on_panic_handler::<log::LmLogger>(info, assert::AssertMode::FatalThrow, results::lib::assert::ResultAssertionFailed::make())
}