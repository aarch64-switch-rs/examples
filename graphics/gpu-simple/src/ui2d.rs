use nx::result::*;
use nx::gpu;
use nx::service::nv;
use nx::arm;

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr;
use core::mem;
use font8x8::UnicodeFonts;

#[derive(Copy, Clone)]
pub struct RGBA8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA8 {
    pub const fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r: r, g: g, b: b, a: a }
    }

    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r: r, g: g, b: b, a: 0xFF }
    }

    const fn decode(raw: u32) -> (u8, u8, u8, u8) {
        let a = (raw & 0xFF) as u8;
        let b = ((raw >> 8) & 0xFF) as u8;
        let c = ((raw >> 16) & 0xFF) as u8;
        let d = ((raw >> 24) & 0xFF) as u8;
        (a, b, c, d)
    }

    pub const fn from_rgba(raw: u32) -> Self {
        let (a, b, g, r) = Self::decode(raw);
        Self::new_rgba(r, g, b, a)
    }

    pub const fn from_abgr(raw: u32) -> Self {
        let (r, g, b, a) = Self::decode(raw);
        Self::new_rgba(r, g, b, a)
    }

    const fn encode(a: u8, b: u8, c: u8, d: u8) -> u32 {
        (a as u32 & 0xFF) | ((b as u32 & 0xFF) << 8) | ((c as u32 & 0xFF) << 16) | ((d as u32 & 0xFF) << 24)
    }

    pub const fn encode_rgba(&self) -> u32 {
        Self::encode(self.a, self.b, self.g, self.r)
    }

    pub const fn encode_abgr(&self) -> u32 {
        Self::encode(self.r, self.g, self.b, self.a)
    }

    const fn blend_color_impl(src: u32, dst: u32, alpha: u32) -> u8 {
        let one_minus_a = 0xFF - alpha;
        ((dst * alpha + src * one_minus_a) / 0xFF) as u8
    }

    pub const fn blend_with(&self, other: Self) -> Self {
        let r = Self::blend_color_impl(other.r as u32, self.r as u32, self.a as u32);
        let g = Self::blend_color_impl(other.g as u32, self.g as u32, self.a as u32);
        let b = Self::blend_color_impl(other.b as u32, self.b as u32, self.a as u32);
        Self::new_rgb(r, g, b)
    }
}

pub struct SurfaceEx<NS: nv::INvDrvService + 'static> {
    gpu_buf: *mut u32,
    gpu_buf_size: usize,
    linear_buf: *mut u32,
    linear_buf_size: usize,
    slot: i32,
    fences: gpu::MultiFence,
    surface_ref: gpu::surface::Surface<NS>,
}

impl<NS: nv::INvDrvService + 'static> SurfaceEx<NS> {
    pub fn from(surface: gpu::surface::Surface<NS>) -> Self {
        let aligned_width = surface.compute_stride() as usize;
        let aligned_height = ((surface.get_height() + 7) & !7) as usize;
        let linear_buf_size = aligned_width * aligned_height;
        unsafe {
            let linear_buf_layout = alloc::alloc::Layout::from_size_align_unchecked(linear_buf_size, 8);
            let linear_buf = alloc::alloc::alloc_zeroed(linear_buf_layout);
            Self { gpu_buf: ptr::null_mut(), gpu_buf_size: 0, linear_buf: linear_buf as *mut u32, linear_buf_size: linear_buf_size, slot: 0, fences: mem::zeroed(), surface_ref: surface }
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let (buf, buf_size, slot, _has_fences, fences) = self.surface_ref.dequeue_buffer(true)?;
        self.gpu_buf = buf as *mut u32;
        self.gpu_buf_size = buf_size;
        self.slot = slot;
        self.fences = fences;
        self.surface_ref.wait_fences(fences, -1)
    }

    fn convert_buffers_gob_impl(out_gob_buf: *mut u8, in_gob_buf: *mut u8, stride: u32) {
        unsafe {
            let mut tmp_out_gob_buf_128 = out_gob_buf as *mut u128;
            for i in 0..32 {
                let y = ((i >> 1) & 0x6) | (i & 0x1);
                let x = ((i << 3) & 0x10) | ((i << 1) & 0x20);
                let out_gob_buf_128 = tmp_out_gob_buf_128 as *mut u128;
                let in_gob_buf_128 = in_gob_buf.offset((y * stride + x) as isize) as *mut u128;
                *out_gob_buf_128 = *in_gob_buf_128;
                tmp_out_gob_buf_128 = tmp_out_gob_buf_128.offset(1);
            }
        }
    }

    fn convert_buffers_impl(out_buf: *mut u8, in_buf: *mut u8, stride: u32, height: u32) {
        let block_height_gobs = 1 << gpu::BLOCK_HEIGHT_LOG2;
        let block_height_px = 8 << gpu::BLOCK_HEIGHT_LOG2;

        let width_blocks = stride >> 6;
        let height_blocks = (height + block_height_px - 1) >> (3 + gpu::BLOCK_HEIGHT_LOG2);
        let mut tmp_out_buf = out_buf;

        for block_y in 0..height_blocks {
            for block_x in 0..width_blocks {
                for gob_y in 0..block_height_gobs {
                    unsafe {
                        let x = block_x * 64;
                        let y = block_y * block_height_px + gob_y * 8;
                        if y < height {
                            let in_gob_buf = in_buf.offset((y * stride + x) as isize);
                            Self::convert_buffers_gob_impl(tmp_out_buf, in_gob_buf, stride);
                        }
                        tmp_out_buf = tmp_out_buf.offset(512);
                    }
                }
            }
        }
    }

    pub fn end(&mut self) -> Result<()> {
        Self::convert_buffers_impl(self.gpu_buf as *mut u8, self.linear_buf as *mut u8, self.surface_ref.compute_stride(), self.surface_ref.get_height());
        arm::cache_flush(self.gpu_buf as *mut u8, self.gpu_buf_size);
        self.surface_ref.queue_buffer(self.slot, self.fences)?;
        self.surface_ref.wait_vsync_event(-1)
    }

    pub fn clear(&mut self, color: RGBA8) {
        unsafe {
            let buf_size_32 = self.linear_buf_size / mem::size_of::<u32>();
            for i in 0..buf_size_32 {
                let cur = self.linear_buf.offset(i as isize);
                *cur = color.encode_abgr();
            }
        }
    }

    pub fn draw_single(&mut self, x: i32, y: i32, color: RGBA8) {
        unsafe {
            let offset = ((self.surface_ref.compute_stride() / mem::size_of::<u32>() as u32) as i32 * y + x) as isize;
            let cur = self.linear_buf.offset(offset);
            let old_color = RGBA8::from_abgr(*cur);
            let new_color = color.blend_with(old_color);
            *cur = new_color.encode_abgr();
        }
    }

    fn clamp(max: i32, value: i32) -> i32 {
        if value < 0 {
            return 0;
        }
        if value > max {
            return max;
        }
        value
    }

    pub fn get_width(&self) -> u32 {
        self.surface_ref.get_width()
    }

    pub fn get_height(&self) -> u32 {
        self.surface_ref.get_height()
    }

    pub fn get_color_format(&self) -> gpu::ColorFormat {
        self.surface_ref.get_color_format()
    }

    pub fn draw(&mut self, x: i32, y: i32, width: i32, height: i32, color: RGBA8) {
        let s_width = self.surface_ref.get_width() as i32;
        let s_height = self.surface_ref.get_height() as i32;
        let x0 = Self::clamp(s_width, x);
        let x1 = Self::clamp(s_width, x + width);
        let y0 = Self::clamp(s_height, y);
        let y1 = Self::clamp(s_height, y + height);
        for y in y0..y1 {
            for x in x0..x1 {
                self.draw_single(x, y, color);
            }
        }
    }

    fn draw_font_text_impl(&mut self, font: &rusttype::Font, text: &str, color: RGBA8, scale: rusttype::Scale, v_metrics: rusttype::VMetrics, x: i32, y: i32) {
        let glyphs: Vec<_> = font.layout(&text[..], scale, rusttype::point(0.0, v_metrics.ascent)).collect();
        for glyph in &glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                // Draw the glyph into the image per-pixel by using the draw closure
                glyph.draw(|g_x, g_y, g_v| {
                    let mut pix_color = color;
                    // Different alpha depending on the pixel
                    pix_color.a = (g_v * 255.0) as u8;
                    self.draw_single(x + g_x as i32 + bounding_box.min.x as i32, y + g_y as i32 + bounding_box.min.y as i32, pix_color);
                });
            }
        }
    }
    
    pub fn draw_font_text(&mut self, font: &rusttype::Font, text: String, color: RGBA8, size: f32, x: i32, y: i32) {
        let scale = rusttype::Scale::uniform(size);
        let v_metrics = font.v_metrics(scale);
        
        let mut tmp_y = y;
        for semi_text in text.lines() {
            self.draw_font_text_impl(font, semi_text, color, scale, v_metrics, x, tmp_y);
            tmp_y += v_metrics.ascent as i32;
        }
    }

    pub fn draw_bitmap_text(&mut self, text: String, color: RGBA8, scale: i32, x: i32, y: i32) {
        let mut tmp_x = x;
        let mut tmp_y = y;
        for c in text.chars() {
            match c {
                '\n' | '\r' => {
                    tmp_y += 8 * scale;
                    tmp_x = x;
                },
                _ => {
                    if let Some(glyph) = font8x8::BASIC_FONTS.get(c) {
                        let char_tmp_x = tmp_x;
                        let char_tmp_y = tmp_y;
                        for gx in &glyph {
                            for bit in 0..8 {
                                match *gx & 1 << bit {
                                    0 => {},
                                    _ => {
                                        self.draw(tmp_x, tmp_y, scale, scale, color);
                                    },
                                }
                                tmp_x += scale;
                            }
                            tmp_y += scale;
                            tmp_x = char_tmp_x;
                        }
                        tmp_x += 8 * scale;
                        tmp_y = char_tmp_y;
                    }
                }
            }
        }
    }
}

impl<NS: nv::INvDrvService> Drop for SurfaceEx<NS> {
    fn drop(&mut self) {
        unsafe {
            let linear_buf_layout = alloc::alloc::Layout::from_size_align_unchecked(self.linear_buf_size, 8);
            alloc::alloc::dealloc(self.linear_buf as *mut u8, linear_buf_layout);
        }
    }
}

// Needed by rusttype

pub trait FloatExt {
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn fract(self) -> Self;
    fn trunc(self) -> Self;
    fn round(self) -> Self;
    fn abs(self) -> Self;
}

impl FloatExt for f32 {
    #[inline]
    fn floor(self) -> Self {
        libm::floorf(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        libm::ceilf(self)
    }
    #[inline]
    fn fract(self) -> Self {
        self - self.trunc()
    }
    #[inline]
    fn trunc(self) -> Self {
        libm::truncf(self)
    }
    #[inline]
    fn round(self) -> Self {
        libm::roundf(self)
    }
    #[inline]
    fn abs(self) -> Self {
        libm::fabsf(self)
    }
}