#![no_std]
#![no_main]

#![feature(thread_local)]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

use nx::svc;
use nx::result::*;
use nx::fs;
use nx::util;
use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::gpu;
use nx::service::hid;
use nx::input;

use core::panic;

extern crate ui2d;

nx::rrt0_define_module_name!("gpu-simple nx example");

//use nx::use_default_allocator;
//use_default_allocator!();

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
    x: u32,
    y: u32,
    size: u32,
    x_incr: i32,
    y_incr: i32,
    x_mult: i32,
    y_mult: i32,
    color: ui2d::RGBA8
}

impl Square {
    pub fn new(x: u32, y: u32, size: u32, color: ui2d::RGBA8) -> Self {
        Self { x, y, size, x_incr: 1, y_incr: 1, x_mult: 1, y_mult: 1, color }
    }

    pub fn handle_render(&mut self, surface: &mut ui2d::SurfaceEx) {
        surface.draw_rect(self.x, self.y, self.size, self.size, self.color, false);

        self.x = self.x.saturating_add_signed(self.x_incr * self.x_mult);
        self.y = self.y.saturating_add_signed(self.y_incr * self.y_mult);

        if self.x == 0 {
            if self.x_incr < 0 {
                self.x_incr = -self.x_incr;
            }
            self.x = 0;
            self.x_mult += 1;
        }
        else if self.x + self.size >= surface.get_width() {
            if self.x_incr > 0 {
                self.x_incr = -self.x_incr;
            }
            self.x = surface.get_width() - self.size;
            self.x_mult += 1;
        }

        if self.y == 0 {
            if self.y_incr < 0 {
                //self.y_incr -= 1;
                self.y_incr = -self.y_incr;
            }
            self.y = 0;
            self.y_mult += 1;
        }
        else if self.y + self.size >= surface.get_height() {
            if self.y_incr > 0 {
                //self.y_incr += 1;
                self.y_incr = -self.y_incr;
            }
            self.y = surface.get_height() - self.size;
            self.y_mult += 1;
        }
    }
}

static LOG_PATH: &str = "sdmc:/fs-test-log.log";

#[no_mangle]
fn main() -> Result<()> {
    fs::initialize_fspsrv_session()?;
    fs::mount_sd_card("sdmc")?;
    nx::service::applet::initialize()?;
    
    let mut log_file = match fs::open_file(LOG_PATH, fs::FileOpenOption::Create() | fs::FileOpenOption::Write() | fs::FileOpenOption::Append()) {
        Ok(f) => f,
        Err(_) => {
            return Ok(());
        }
    };

    let mut gpu_ctx = match gpu::Context::new(gpu::NvDrvServiceKind::Applet, gpu::ViServiceKind::System, 0x800000) {
        Ok(ok) => ok,
        Err(e) => {
            let message = ::alloc::format!("Failed to get gpu context. Error {} - {}\n", e.get_value(), e.get_description());
            let _ = log_file.write_array(message.as_bytes());
            return Ok(());
        }
    };

    let supported_style_tags = hid::NpadStyleTag::Handheld() | hid::NpadStyleTag::FullKey()| hid::NpadStyleTag::JoyDual() | hid::NpadStyleTag::JoyLeft() | hid::NpadStyleTag::JoyRight();
    let input_ctx = match input::Context::new(supported_style_tags, 2) {
        Ok(ok) => ok,
        Err(e) => {
            let message = ::alloc::format!("Failed to get input context. Error {}\n", e);
            let _ = log_file.write_array(message.as_bytes());
            return Ok(());
        }
    };

    

    let color_fmt = gpu::ColorFormat::A8B8G8R8;
    let mut squares: Vec<Square> = Vec::new();

    let c_white = ui2d::RGBA8::new_rgb(0xFF, 0xFF, 0xFF);
    let c_black = ui2d::RGBA8::new_rgb(0, 0, 0);
    let c_royal_blue = ui2d::RGBA8::new_rgb(65, 105, 225);

    let font = match ui2d::FontType::try_from_slice(include_bytes!("../../font/Roboto-Medium.ttf")){
        Ok(ok) => ok,
        Err(_) => {
            let _ = log_file.write_array(b"Failed to parse font");
            return Ok(());
        }
    };
   
    let mut surface = ui2d::SurfaceEx::from(
        match gpu_ctx.create_stray_layer_surface("Default", 2, Default::default(), color_fmt, gpu::PixelFormat::RGBA_8888) {
            Ok(ok) => ok,
            Err(e) => {
                let message = ::alloc::format!("Failed to surface handle. Error {} - {}\n", e.get_value(), e.get_description());
                let _ = log_file.write_array(message.as_bytes());
                return Ok(());
            }
        }
    );

    'render: loop {

        for controller in [hid::NpadIdType::Handheld, hid::NpadIdType::No1].iter().cloned() {
            let mut p_handheld = input_ctx.get_player(controller);

            let buttons_down = p_handheld.get_buttons_down();
            if buttons_down.contains(hid::NpadButton::A()) {
                squares.push(Square::new(10, 10, 50, c_royal_blue));
            }
            else if buttons_down.contains(hid::NpadButton::B()) {
                squares.pop();
            }
            else if buttons_down.contains(hid::NpadButton::Plus()) {
                // Exit if Plus/+ is pressed.
                break 'render;
            }
        }


        match surface.start() {
            Ok(ok) => ok,
            Err(e) => {
                let message = ::alloc::format!("Failed to get start frame rendering. Error {} - {}\n", e.get_value(), e.get_description());
                let _ = log_file.write_array(message.as_bytes());
                return Ok(());
            }
        };
        
        surface.clear(c_white);
        surface.draw_font_text(&font, String::from("(Drawn with Roboto TTF font)\n\nHello world from aarch64-switch-rs!\nPress A to spawn moving squares. Press B to remove the most recently spawned square\nPress + to exit this test."), c_black, 15.0, 10, 10, true);
        surface.draw_bitmap_text(String::from("(Drawn with standard bitmap font)\n\nHello world from aarch64-switch-rs!\nPress A to spawn moving squares.\nPress + to exit this test."), c_black, 2, 10, 250, true);

        for square in squares.iter_mut() {
            square.handle_render(&mut surface);
        }

        match surface.end(){
            Ok(ok) => ok,
            Err(e) => {
                let message = ::alloc::format!("Failed to get end frame rendering. Error {} - {}\n", e.get_value(), e.get_description());
                let _ = log_file.write_array(message.as_bytes());
                return Ok(());
            }
        };
    }


    let _ = log_file.write_array(b"exiting normally\n");

    Ok(()) 
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
