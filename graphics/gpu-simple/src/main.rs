#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

extern crate nx;
use nx::svc;
use nx::result::*;
use nx::util;
use nx::diag::abort;
use nx::diag::log;
use nx::gpu;
use nx::service::hid;
use nx::input;

use core::panic;

extern crate ui2d;

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

pub struct Square {
    x: i32,
    y: i32,
    size: i32,
    x_incr: i32,
    y_incr: i32,
    x_mult: i32,
    y_mult: i32,
    color: ui2d::RGBA8
}

impl Square {
    pub fn new(x: i32, y: i32, size: i32, color: ui2d::RGBA8) -> Self {
        Self { x: x, y: y, size: size, x_incr: 1, y_incr: 1, x_mult: 1, y_mult: 1, color: color }
    }

    pub fn handle_render(&mut self, surface: &mut ui2d::SurfaceEx) {
        surface.draw(self.x, self.y, self.size, self.size, self.color, false);

        self.x += self.x_incr * self.x_mult;
        self.y += self.y_incr * self.y_mult;

        if self.x <= 0 {
            if self.x_incr < 0 {
                self.x_incr -= 1;
                self.x_incr = -self.x_incr;
            }
            self.x += self.x_incr * self.x_mult;
            self.x_mult += 1;
        }
        else if (self.x + self.size) as u32 >= surface.get_width() {
            if self.x_incr > 0 {
                self.x_incr += 1;
                self.x_incr = -self.x_incr;
            }
            self.x += self.x_incr * self.x_mult;
            self.x_mult += 1;
        }

        if self.y <= 0 {
            if self.y_incr < 0 {
                self.y_incr -= 1;
                self.y_incr = -self.y_incr;
            }
            self.y += self.y_incr * self.y_mult;
            self.y_mult += 1;
        }
        else if (self.y + self.size) as u32 >= surface.get_height() {
            if self.y_incr > 0 {
                self.y_incr += 1;
                self.y_incr = -self.y_incr;
            }
            self.y += self.y_incr * self.y_mult;
            self.y_mult += 1;
        }
    }
}

#[no_mangle]
pub fn main() -> Result<()> {
    let mut gpu_ctx = gpu::Context::new(gpu::NvDrvServiceKind::Applet, gpu::ViServiceKind::System, 0x800000)?;

    let supported_style_tags = hid::NpadStyleTag::Handheld();
    let supported_npad_ids = [hid::NpadIdType::Handheld];
    let input_ctx = input::Context::new(supported_style_tags, &supported_npad_ids)?;

    let color_fmt = gpu::ColorFormat::A8B8G8R8;
    let mut squares: Vec<Square> = Vec::new();

    let c_white = ui2d::RGBA8::new_rgb(0xFF, 0xFF, 0xFF);
    let c_black = ui2d::RGBA8::new_rgb(0, 0, 0);
    let c_royal_blue = ui2d::RGBA8::new_rgb(65, 105, 225);

    let font_data = include_bytes!("../../font/Roboto-Medium.ttf");
    let font = ui2d::Font::try_from_bytes(font_data as &[u8]).unwrap();

    let mut surface = ui2d::SurfaceEx::from(gpu_ctx.create_stray_layer_surface("Default", 2, color_fmt, gpu::PixelFormat::RGBA_8888, gpu::Layout::BlockLinear)?);

    loop {
        let mut p_handheld = input_ctx.get_player(hid::NpadIdType::Handheld);

        let buttons_down = p_handheld.get_buttons_down();
        if buttons_down.contains(hid::NpadButton::A()) {
            squares.push(Square::new(10, 10, 50, c_royal_blue));
        }
        else if buttons_down.contains(hid::NpadButton::Plus()) {
            // Exit if Plus/+ is pressed.
            break;
        }

        surface.start()?;
        
        surface.clear(c_white);
        surface.draw_font_text(&font, String::from("(Drawn with Roboto TTF font)\n\nHello world from aarch64-switch-rs!\nPress A to spawn moving squares.\nPress + to exit this test."), c_black, 25.0, 10, 10, true);
        surface.draw_bitmap_text(String::from("(Drawn with standard bitmap font)\n\nHello world from aarch64-switch-rs!\nPress A to spawn moving squares.\nPress + to exit this test."), c_black, 2, 10, 250, true);

        for square in squares.iter_mut() {
            square.handle_render(&mut surface);
        }

        surface.end()?;
    }

    Ok(())
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<log::LmLogger>(info, abort::AbortLevel::FatalThrow())
}