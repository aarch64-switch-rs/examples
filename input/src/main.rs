#![no_std]
#![no_main]

extern crate nx;

extern crate alloc;

use nx::result::*;
use nx::util;
use nx::svc;
use nx::input;
use nx::service::hid;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;

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
    // Support all basic controller styles (pro-controller, handheld and joy-cons in single and dual modes)
    let supported_style_tags = hid::NpadStyleTag::FullKey() | hid::NpadStyleTag::Handheld() | hid::NpadStyleTag::JoyDual() | hid::NpadStyleTag::JoyLeft() | hid::NpadStyleTag::JoyRight();

    let input_ctx = input::Context::new(supported_style_tags, 1)?;

    // Track player 1 and handheld
    let mut p1 = input_ctx.get_player(hid::NpadIdType::No1);
    let mut ph = input_ctx.get_player(hid::NpadIdType::Handheld);

    loop {
        // If A gets pressed as player-1 or X gets pressed as handheld, exit
        let p1_down = p1.get_buttons_down();
        let ph_down = ph.get_buttons_down();

        if p1_down.contains(hid::NpadButton::A()) {
            break;
        }
        if ph_down.contains(hid::NpadButton::X()) {
            break;
        }

        // Sleep 10ms (aka 10'000'000 ns)
        svc::sleep_thread(10_000_000)?;
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}