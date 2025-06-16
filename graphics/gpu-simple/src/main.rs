#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nx::diag::abort;
use nx::diag::log::lm::LmLogger;
use nx::fs;
use nx::fs::mount_sd_card;
use nx::fs::FileOpenOption;
use nx::gpu;
use nx::gpu::canvas::AlphaBlend;
use nx::gpu::canvas::BufferedCanvas;
use nx::gpu::canvas::Canvas;
use nx::gpu::canvas::RGBA8;
use nx::gpu::surface::Surface;
use nx::input;
use nx::service::hid;
use nx::svc;
use nx::sync::RwLock;
use nx::util;

use core::panic;

nx::rrt0_define_module_name!("gpu-simple");

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

pub struct Square {
    x: u32,
    y: u32,
    size: u32,
    x_incr: i32,
    y_incr: i32,
    x_mult: i32,
    y_mult: i32,
    color: RGBA8,
}

impl Square {
    pub fn new(x: u32, y: u32, size: u32, color: RGBA8) -> Self {
        Self {
            x,
            y,
            size,
            x_incr: 1,
            y_incr: 1,
            x_mult: 1,
            y_mult: 1,
            color,
        }
    }

    pub fn tick(&mut self, surface: &Surface) {
        self.x = self.x.saturating_add_signed(self.x_incr * self.x_mult);
        self.y = self.y.saturating_add_signed(self.y_incr * self.y_mult);

        if self.x == 0 {
            if self.x_incr < 0 {
                self.x_incr = -self.x_incr;
            }
            self.x = 0;
            self.x_mult += 1;
        } else if self.x + self.size >= surface.width() {
            if self.x_incr > 0 {
                self.x_incr = -self.x_incr;
            }
            self.x = surface.width() - self.size;
            self.x_mult += 1;
        }

        if self.y == 0 {
            if self.y_incr < 0 {
                //self.y_incr -= 1;
                self.y_incr = -self.y_incr;
            }
            self.y = 0;
            self.y_mult += 1;
        } else if self.y + self.size >= surface.height() {
            if self.y_incr > 0 {
                //self.y_incr += 1;
                self.y_incr = -self.y_incr;
            }
            self.y = surface.height() - self.size;
            self.y_mult += 1;
        }
    }
    pub fn render(&self, surface: &mut BufferedCanvas<'_, RGBA8>) {
        
        surface.draw_rect(self.x as i32, self.y as i32, self.size, self.size, self.color, gpu::canvas::AlphaBlend::None);
    }
}

#[no_mangle]
fn main() {

    fs::initialize_fspsrv_session().unwrap();
    mount_sd_card("sdmc").unwrap();
    let mut log_file = fs::open_file("sdmc:/gpu-simple.log", FileOpenOption::Append() | FileOpenOption::Create() | FileOpenOption::Write()).unwrap();

    let supported_style_tags = hid::NpadStyleTag::Handheld()
        | hid::NpadStyleTag::FullKey()
        | hid::NpadStyleTag::JoyDual()
        | hid::NpadStyleTag::JoyLeft()
        | hid::NpadStyleTag::JoyRight();
    let input_ctx = match input::Context::new(supported_style_tags, 2) {
        Ok(ok) => ok,
        Err(e) => {
            let _ = log_file.write_array(format!("Error getting input context: {:#X}", e.get_value()).as_bytes());
            return;
        }
    };

    let mut squares: Vec<Square> = Vec::new();

    let c_white = RGBA8::new_scaled(0xFF, 0xFF, 0xFF, 0xFF);
    let c_black = RGBA8::new_scaled(0, 0, 0, 255);
    let c_royal_blue = RGBA8::new_scaled(65, 105, 225, 255);

    let font =
        match nx::gpu::canvas::Font::try_from_slice(include_bytes!("../../font/Roboto-Medium.ttf"))
        {
            Ok(ok) => ok,
            Err(_e) => {
                let _ = log_file.write_array("Error getting font: invalid_font".as_bytes());
                return;
            }
        };

    let mut canvas_manager = {
        let gpu_ctx = match gpu::Context::new(
            gpu::NvDrvServiceKind::Applet,
            gpu::ViServiceKind::System,
            0x800000,
        ) {
            Ok(ok) => ok,
            Err(e) => {
                let _ = log_file.write_array(format!("Error getting gpu context: {:#X}", e.get_value()).as_bytes());
                return;
            }
        };

        match nx::gpu::canvas::CanvasManager::new_stray(
        alloc::sync::Arc::new(RwLock::new(gpu_ctx)),
        Default::default(),
        3,
        gpu::BlockLinearHeights::FourGobs,
    ) {
        Ok(s) => s,
        Err(e) => {
            let _ = log_file.write_array(format!("Error getting canvas manager: {:#X}", e.get_value()).as_bytes());
            return;
    }}};

    'render: loop {

        let mut stick_statuses = Vec::new();
        for controller in [hid::NpadIdType::Handheld, hid::NpadIdType::No1 ]
            .iter()
            .cloned()
        {
            let mut p_handheld = input_ctx.get_player(controller);

            let buttons_down = p_handheld.get_buttons_down();
            if buttons_down.contains(hid::NpadButton::A()) {
                squares.push(Square::new(10, 10, 50, c_royal_blue));
            } else if buttons_down.contains(hid::NpadButton::B()) {
                squares.pop();
            } else if buttons_down.contains(hid::NpadButton::Plus()) {
                // Exit if Plus/+ is pressed.
                break 'render;
            }

            stick_statuses.push(format!("{:?}, {:?}, {:?}", controller, p_handheld.get_controller_type(), p_handheld.get_stick_status(p_handheld.get_reported_style_tag())));
        }

        for square in squares.iter_mut() {
            square.tick(&canvas_manager.surface);
        }

        let font = font.clone();
        let squares_ref = &squares;
        let _ = canvas_manager.render(Some(c_white), move |surface| {
            surface.draw_font_text(&font.clone(), String::from("(Drawn with Roboto TTF font)\n\nHello world from aarch64-switch-rs!\nPress A to spawn moving squares. Press B to remove the most recently spawned square\nPress + to exit this test."), c_black, 15.0, 10, 10, AlphaBlend::Source);
            //surface.draw_ascii_bitmap_text(String::from("(Drawn with standard bitmap font)\n\nHello world from aarch64-switch-rs!\nPress A to spawn moving squares.\nPress + to exit this test."), c_black, 2, 10, 250, AlphaBlend::None);
            
            for (index, s) in stick_statuses.iter().enumerate() {
                surface.draw_ascii_bitmap_text(s.as_str(), c_black, 1, 20, 250 + 10 * index as i32, AlphaBlend::None);
            }

            for square in squares_ref {
                square.render(surface);
            }

            surface.draw_circle_filled(500,400, 200, c_royal_blue, AlphaBlend::None);
            surface.draw_circle(500, 400, 250, 3, RGBA8::new_scaled(0xff, 0, 0, 0xff), AlphaBlend::None);

            Ok(())
        });

        let _ = canvas_manager.wait_vsync_event(None);
    }

}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
