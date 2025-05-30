#![no_std]
#![no_main]

extern crate alloc;

use ::alloc::format;

use alloc::sync::Arc;
use nx::diag::abort;
use nx::fs::FileOpenOption;
use nx::gpu::canvas::Canvas as _;
use nx::gpu::{BlockLinearHeights, SCREEN_HEIGHT, SCREEN_WIDTH};
use nx::input;
use nx::ipc::sf::AppletResourceUserId;
use nx::result::*;
use nx::service::hid;
use nx::service::vi::LayerFlags;
use nx::sync::RwLock;
use nx::util;
use nx::{fs, gpu};

use core::panic;


// We're using 8MB of heap
const CUSTOM_HEAP_LEN: usize = 0x800000;
static mut CUSTOM_HEAP: [u8; CUSTOM_HEAP_LEN] = [0; CUSTOM_HEAP_LEN];

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub fn initialize_heap(_hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    unsafe { util::PointerAndSize::new(&raw mut CUSTOM_HEAP as *mut _, CUSTOM_HEAP.len()) }
}

static LOG_PATH: &str = "sdmc:/fs-test-log.log";

type RGBType = nx::gpu::canvas::RGBA4;
const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;
const BLOCK_HEIGHT_CONFIG: BlockLinearHeights = gpu::BlockLinearHeights::TwoGobs;
const BUFFER_COUNT: u32 = 2;

//#[no_mangle]
//static HEAP_SIZE: usize = CanvasManager::<RGBType>::total_heap_required(WIDTH, HEIGHT, BLOCK_HEIGHT_CONFIG, BUFFER_COUNT);

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

#[unsafe(no_mangle)]
pub fn main() -> Result<()> {
    /*let wait = core::sync::atomic::AtomicBool::new(true);
    while wait.load(core::sync::atomic::Ordering::Relaxed) {
        sleep(100);
    }*/

    fs::initialize_fspsrv_session()?;
    fs::mount_sd_card("sdmc")?;

    let mut file = fs::open_file(
        LOG_PATH,
        FileOpenOption::Create() | FileOpenOption::Write() | FileOpenOption::Append(),
    )?;

    let gpu_ctx = Arc::new(RwLock::new(
        match gpu::Context::new(
            gpu::NvDrvServiceKind::Applet,
            gpu::ViServiceKind::Manager,
            0x40000,
        ) {
            Ok(ok) => ok,
            Err(e) => {
                let _ = file.write_array(format!("Error getting gpu context: {}\n", e).as_bytes());
                return Ok(());
            }
        },
    ));

    let supported_tags =
        hid::NpadStyleTag::Handheld() | hid::NpadStyleTag::FullKey() | hid::NpadStyleTag::JoyDual();
    let input_ctx = match input::Context::new(supported_tags, 1) {
        Ok(ok) => ok,
        Err(e) => {
            let _ = file.write_array(format!("Error getting input context: {}\n", e).as_bytes());
            return Ok(());
        }
    };

    let x = (SCREEN_WIDTH - WIDTH) / 2;
    let y = (SCREEN_HEIGHT - HEIGHT) / 2;

    let _c_empty = RGBType::new_scaled(0, 0, 0, 0);
    let _c_white = RGBType::new_scaled(0xFF, 0xFF, 0xFF, 0xFF);
    let _c_black = RGBType::new_scaled(0, 0, 0, 0xFF);
    let _c_royal_blue = RGBType::new_scaled(65, 105, 225, 0xFF);

    let mut layer_visible = true;
    let mut surface = match nx::gpu::canvas::CanvasManager::new_managed(
        gpu_ctx,
        None,
        x,
        y,
        gpu::LayerZ::Max,
        WIDTH,
        HEIGHT,
        AppletResourceUserId::new(0),
        LayerFlags::None(),
        BUFFER_COUNT,
        BLOCK_HEIGHT_CONFIG,
        gpu::surface::ScaleMode::PreseveAspect { height: 1080 },
    ) {
        Ok(ok) => ok,
        Err(e) => {
            let _ = file.write_array(format!("Error getting surface: {}\n", e).as_bytes());
            return Ok(());
        }
    };

    let mut offset = 0;
    'render: loop {
        for controller in [hid::NpadIdType::Handheld, hid::NpadIdType::No1]
            .iter()
            .cloned()
        {
            let mut p_handheld = input_ctx.get_player(controller);

            let buttons_down = p_handheld.get_buttons_down();
            if buttons_down.contains(hid::NpadButton::X()) {
                layer_visible = !layer_visible;
                if let Err(e) = surface.surface.set_visible(layer_visible) {
                    let _ = file.write_array(
                        format!(
                            "Error setting surface visibility to {}: {}\n",
                            layer_visible, e
                        )
                        .as_bytes(),
                    );
                }
            } else if buttons_down.contains(hid::NpadButton::Plus()) {
                // Exit if Plus/+ is pressed.
                break 'render;
            }
        }

        let _ = surface.render_unbuffered(Some(_c_black), |canvas| {
            canvas.draw_rect(
                offset % 256,
                0,
                10,
                canvas.height(),
                RGBType::new_scaled(255, 0, 0, 128),
                gpu::canvas::AlphaBlend::None,
            );
            canvas.draw_rect(
                0,
                offset % 256,
                canvas.width(),
                10,
                RGBType::new_scaled(0, 255, 0, 128),
                gpu::canvas::AlphaBlend::Destination,
            );

            Ok(())
        });
        let _ = surface.wait_vsync_event(None);
        offset += 1;
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    let _panic_str = format!("{}", info);
    nx::diag::abort::abort(
        abort::AbortLevel::FatalThrow(),
        nx::rc::ResultPanicked::make(),
    );
}
