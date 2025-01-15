#![no_std]
#![no_main]

extern crate alloc;

extern crate nx;
use nx::service::applet::ILibraryAppletAccessor;
use nx::service::applet::ILibraryAppletCreator;
use nx::service::applet::ILibraryAppletProxy;
use nx::service::applet::IStorage;
use nx::service::applet::IStorageAccessor;
use nx::svc;
use nx::arm;
use nx::result::*;
use nx::util;
use nx::wait;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::ipc::sf;
use nx::service;
use nx::service::applet;
use nx::service::applet::IAllSystemAppletProxiesService;

use core::panic;

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    }
    else {
        let heap_size: usize = 0x800000;
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

#[no_mangle]
pub fn main() -> Result<()> {
    let applet_proxy_srv = service::new_service_object::<applet::AllSystemAppletProxiesService>()?;
    
    let attr: applet::AppletAttribute = unsafe { core::mem::zeroed() };
    let lib_applet_proxy = applet_proxy_srv.open_library_applet_proxy(sf::ProcessId::new(), sf::Handle::from(svc::CURRENT_PROCESS_PSEUDO_HANDLE), sf::Buffer::from_var(&attr))?;
    let lib_applet_creator = lib_applet_proxy.get_library_applet_creator()?;
    let lib_applet_accessor = lib_applet_creator.create_library_applet(applet::AppletId::LibraryAppletPlayerSelect, applet::LibraryAppletMode::AllForeground)?;

    {
        let common_args = CommonArguments::new(1, 0x20000, 0, false);
        let storage = lib_applet_creator.create_storage(common_args.size as usize)?;
        {
            let storage_accessor = storage.open()?;
            storage_accessor.write(0, sf::Buffer::from_other_var(&common_args))?;
        }
        lib_applet_accessor.push_in_data(storage)?;
    }

    {
        let mut data: [u8; 0xA0] = [0; 0xA0];
        data[0x96] = 1;
        let storage = lib_applet_creator.create_storage(data.len())?;
        {
            let storage_accessor = storage.open()?;
            storage_accessor.write(0, sf::Buffer::from_array(&data))?;
        }
        lib_applet_accessor.push_in_data(storage)?;
    }

    let event_handle = lib_applet_accessor.get_applet_state_changed_event()?;
    lib_applet_accessor.start()?;

    wait::wait_handles(&[event_handle.handle], -1)?;

    svc::close_handle(event_handle.handle)?;

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}