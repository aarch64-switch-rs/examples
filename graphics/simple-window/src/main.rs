#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

extern crate nx;
use nx::ipc::sf::AppletResourceUserId;
use nx::result::*;
use nx::util;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::gpu;
use nx::service::vi;
use nx::service::hid;
use nx::input;

use core::panic;
use core::sync::atomic::AtomicBool;

extern crate ui2d;

// We're using 8MB of heap
const CUSTOM_HEAP_LEN: usize = 0x800000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_LEN] = [0; CUSTOM_HEAP_LEN];

#[no_mangle]
#[allow(static_mut_refs)]
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
    let wait = AtomicBool::new(true);
    while wait.load(core::sync::atomic::Ordering::Relaxed) {
        let _ = nx::thread::sleep(100_000);
    }

    let mut gpu_ctx = gpu::Context::new(gpu::NvDrvServiceKind::Applet, gpu::ViServiceKind::Manager, 0x40000)?;

    let supported_tags = hid::NpadStyleTag::Handheld();
    let input_ctx = input::Context::new(supported_tags, 1)?;

    let width: u32 = 200;
    let height: u32 = 200;
    let x = 0.0;//((1280 - width) / 2) as f32;
    let y = 0.0;//((720 - height) / 2) as f32;
    let color_fmt = gpu::ColorFormat::A8B8G8R8;

    let c_empty = ui2d::RGBA8::new_rgba(0, 0, 0, 0);
    let c_white = ui2d::RGBA8::new_rgb(0xFF, 0xFF, 0xFF);
    let c_black = ui2d::RGBA8::new_rgb(0, 0, 0);
    let c_royal_blue = ui2d::RGBA8::new_rgb(65, 105, 225);

    let font = ui2d::FontType::try_from_slice(include_bytes!("../../font/Roboto-Medium.ttf")).unwrap();

    let mut layer_visible = true;
    let gpu_ctx = gpu_ctx.create_managed_layer_surface("Default", AppletResourceUserId::from_global(), vi::LayerFlags::None(), x, y, width, height, Default::default(), gpu::LayerZ::Max, 2, color_fmt, gpu::PixelFormat::RGBA_8888)?;
    let mut surface = ui2d::SurfaceEx::from(gpu_ctx);

    'render: loop {
        for controller in [hid::NpadIdType::Handheld, hid::NpadIdType::No1].iter().cloned() {
            let mut p_handheld = input_ctx.get_player(controller);

            let buttons_down = p_handheld.get_buttons_down();
            if buttons_down.contains(hid::NpadButton::X()) {
                layer_visible = !layer_visible;
            }
            else if buttons_down.contains(hid::NpadButton::Plus()) {
                // Exit if Plus/+ is pressed.
                break 'render;
            }
        }
        
        surface.start()?;
        if layer_visible {
            surface.clear(c_white);
            surface.draw_font_text(&font, "Hello!", c_black, 25.0, 0, 10, true);
            surface.draw_bitmap_text("Hello bmt!", c_royal_blue, 2, 0, 50, true);
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
    let panic_str = format!("{}", info);
    nx::diag::abort::abort(abort::AbortLevel::FatalThrow(), nx::rc::ResultPanicked::make());
    //util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}