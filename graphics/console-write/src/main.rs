#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;

use nx::console::scrollback::ScrollbackConsole;
use nx::diag::abort;
use nx::fs;
use nx::fs::mount_sd_card;
use nx::fs::FileOpenOption;
use nx::gpu;
use nx::input;
use nx::rand::RandomService;
use nx::rand::Rng;
use nx::result::*;
use nx::service::hid;
use nx::service::new_service_object;
use nx::svc;
use nx::sync::RwLock;
use nx::thread;
use nx::util;

use core::num::NonZeroU16;
use core::panic;

nx::rrt0_define_module_name!("console-write");

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    } else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[no_mangle]
fn main() {
    let mut console: ScrollbackConsole = {
        let gpu_ctx = match gpu::Context::new(
            gpu::NvDrvServiceKind::Applet,
            gpu::ViServiceKind::System,
            0x40000,
        ) {
            Ok(ctx) => ctx,
            Err(e) => panic!("{}", e),
        };

        match ScrollbackConsole::new(
            Arc::new(RwLock::new(gpu_ctx)),
            300,
            NonZeroU16::new(90).unwrap(),
            true,
            None,
            2
        ){
            Ok(ctx) => ctx,
            Err(e) => panic!("{}", e)
        }
    };

    fs::initialize_fspsrv_session().unwrap();
    mount_sd_card("sdmc").unwrap();
    let mut text_file = fs::open_file("sdmc:/lorem_ipsum", FileOpenOption::Read()).unwrap();

    let supported_style_tags = hid::NpadStyleTag::Handheld()
        | hid::NpadStyleTag::FullKey()
        | hid::NpadStyleTag::JoyDual()
        | hid::NpadStyleTag::JoyLeft()
        | hid::NpadStyleTag::JoyRight();
    let input_ctx = input::Context::new(supported_style_tags, 2).unwrap();

    let mut rand = new_service_object::<nx::rand::RandomService>().unwrap();

    'render: loop {
        for controller in [hid::NpadIdType::Handheld, hid::NpadIdType::No1]
            .iter()
            .cloned()
        {
            let mut p_handheld = input_ctx.get_player(controller);

            let buttons_down = p_handheld.get_buttons_down();
            if buttons_down.contains(hid::NpadButton::Down()) {
                console.scroll_down();
            } else if buttons_down.contains(hid::NpadButton::Up()) {
                console.scroll_up();
            } else if buttons_down.contains(hid::NpadButton::Plus()) {
                // Exit if Plus/+ is pressed.
                break 'render;
            }
        }

        let mut read_buf = vec![0u8; <RandomService as Rng>::random_range(&mut rand, 1..8)];
        match text_file.read_array(read_buf.as_mut_slice()) {
            Ok(read_size) => read_buf.truncate(read_size),
            Err(_) => {
                continue;
            }
        }

        let push_str = String::from_utf8(read_buf).unwrap();

        console.write(push_str);

        console.draw().unwrap();

        let _ = console.wait_vsync_event(None);

        let _ = thread::sleep(<RandomService as Rng>::random_range(&mut rand, 100..100000));
    }
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    let _info_message = format!("{}", info);
    nx::diag::abort::abort(abort::AbortLevel::Panic(), nx::rc::ResultPanicked::make());
}
