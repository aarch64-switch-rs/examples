#![no_std]
#![no_main]

extern crate alloc;

extern crate nx;
use nx::arm;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::ipc::sf;
use nx::service::applet;
use nx::service::applet::ILibraryAppletAccessorClient;
use nx::service::applet::ILibraryAppletCreatorClient;
use nx::service::applet::IStorageClient;
use nx::service::applet::IStorageAccessorClient;
use nx::service::applet::ProxyCommon as _;
use nx::svc;
use nx::util;
use nx::wait;

use core::ops::Deref;
use core::panic;

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    } else {
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
    pub system_tick: u64,
}

impl CommonArguments {
    pub fn new(
        version: u32,
        la_api_version: u32,
        theme_color: u32,
        play_startup_sound: bool,
    ) -> Self {
        Self {
            version: version,
            size: core::mem::size_of::<Self>() as u32,
            la_api_version: la_api_version,
            theme_color: theme_color,
            play_startup_sound: play_startup_sound,
            pad: [0; 7],
            system_tick: arm::get_system_tick(),
        }
    }
}

#[no_mangle]
pub fn main() {
    applet::initialize().expect("Applet initialization failed");
    
    let lib_applet_proxy_guard = applet::get_applet_proxy();
    let lib_applet_proxy = lib_applet_proxy_guard.deref().as_ref().expect("Error unwrapping applet proxy after successful init");
    let lib_applet_creator = lib_applet_proxy.get_library_applet_creator().expect("Error creating a library applet creator");
    let mut lib_applet_accessor = lib_applet_creator.create_library_applet(
        applet::AppletId::LibraryAppletPlayerSelect,
        applet::LibraryAppletMode::AllForeground,
    ).expect("Error creating library applet accessor");

    {
        let common_args = CommonArguments::new(1, 0x20000, 0, false);
        let storage = lib_applet_creator.create_storage(common_args.size as usize).expect("Error creating storage");
        {
            let storage_accessor = storage.open().expect("Failed to open storage");
            storage_accessor.write(0, sf::Buffer::from_other_var(&common_args)).expect("Error writing to storage");
        }
        lib_applet_accessor.push_in_data(storage).expect("Failed to add data to storage");
    }

    {
        let mut data: [u8; 0xA0] = [0; 0xA0];
        data[0x96] = 1;
        let storage = lib_applet_creator.create_storage(data.len()).expect("Error openning storage 2");
        {
            let storage_accessor = storage.open().expect("Failed to open storage 2");
            storage_accessor.write(0, sf::Buffer::from_array(&data)).expect("Error writing to storage 2");
        }
        lib_applet_accessor.push_in_data(storage).expect("failed to add data to storage 2");
    }

    let event_handle = lib_applet_accessor.get_applet_state_changed_event().expect("Error creating applet listener");
    lib_applet_accessor.start().expect("failed to start applet accessor.");

    wait::wait_handles(&[event_handle.handle], -1).expect("Error waiting for the applet to close");

    let _ = svc::close_handle(event_handle.handle);

}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
