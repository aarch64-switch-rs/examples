#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;

extern crate nx;
use nx::result::*;
use nx::util;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::gpu;
use nx::service::vi;
use nx::service::hid;
use nx::input;

use core::panic;

extern crate ui2d;

// We're using 8MB of heap
const CUSTOM_HEAP_LEN: usize = 0x800000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_LEN] = [0; CUSTOM_HEAP_LEN];

#[no_mangle]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe {
        util::PointerAndSize::new(CUSTOM_HEAP.as_mut_ptr(), CUSTOM_HEAP.len())
    }
}

/*
fn draw_circle(surface: &mut ui2d::SurfaceEx, x: i32, y: i32, radius: u32, color: ui2d::RGBA8, blend: bool) {
    let pi: f64 = 3.1415926535;
    let mut i: f64 = 0.0;
    while i < 360.0 {
        let x1 = radius as f64 * libm::cos(i * pi / 180.0);
        let y1 = radius as f64 * libm::sin(i * pi / 180.0);
        surface.draw(x + (x1 as i32), y + (y1 as i32), 1, 1, color, blend);
        i += 0.1;
    }
}
*/

#[no_mangle]
pub fn main() -> Result<()> {
    let mut gpu_ctx = gpu::Context::new(gpu::NvDrvServiceKind::Applet, gpu::ViServiceKind::Manager, 0x40000)?;

    let supported_tags = hid::NpadStyleTag::Handheld();
    let supported_npad_id = hid::NpadIdType::Handheld;
    let input_ctx = input::Context::new(supported_tags, &[supported_npad_id])?;

    let width: u32 = 500;
    let height: u32 = 500;
    let x = ((1280 - width) / 2) as f32;
    let y = ((720 - height) / 2) as f32;
    let color_fmt = gpu::ColorFormat::A8B8G8R8;

    let c_empty = ui2d::RGBA8::new_rgba(0, 0, 0, 0);
    let c_white = ui2d::RGBA8::new_rgb(0xFF, 0xFF, 0xFF);
    let c_black = ui2d::RGBA8::new_rgb(0, 0, 0);
    let c_royal_blue = ui2d::RGBA8::new_rgb(65, 105, 225);

    let font_data = include_bytes!("../../font/Roboto-Medium.ttf");
    let font = ui2d::Font::try_from_bytes(font_data as &[u8]).unwrap();

    let mut layer_visible = true;
    let mut surface = ui2d::SurfaceEx::from(gpu_ctx.create_managed_layer_surface("Default", 0, vi::LayerFlags::None(), x, y, width, height, gpu::LayerZ::Max, 2, color_fmt, gpu::PixelFormat::RGBA_8888, gpu::Layout::BlockLinear)?);

    loop {
        let mut p_handheld = input_ctx.get_player(supported_npad_id);

        let buttons_down = p_handheld.get_buttons_down();
        if buttons_down.contains(hid::NpadButton::X()) {
            layer_visible = !layer_visible;
        }
        else if buttons_down.contains(hid::NpadButton::Plus()) {
            // Exit if Plus/+ is pressed
            break;
        }
        
        surface.start()?;
        if layer_visible {
            surface.clear(c_white);
            surface.draw_font_text(&font, String::from("Hello!"), c_black, 25.0, 10, 10, true);
            surface.draw_bitmap_text(String::from("Hello!"), c_royal_blue, 2, 10, 250, true);
        }
        else {
            surface.clear(c_empty);
        }
        surface.end()?;
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}