use std::fmt::Debug;
use std::slice;

use glam::{Vec2, Vec4};

use crate::convert::cast_usize;

#[derive(Debug, PartialEq, Clone)]
pub struct Image {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Image {
        let w = cast_usize(width);
        let h = cast_usize(height);

        Image {
            width: w,
            height: h,
            buffer: vec![0; w * h],
        }
    }

    pub fn from_pixel_rgba(width: u32, height: u32, pixel: [u8; 4]) -> Image {
        let mut image = Image::new(width, height);

        let pixel_u32 = u32::from_le_bytes(pixel);
        for p in image.buffer.iter_mut() {
            *p = pixel_u32;
        }

        image
    }

    pub fn from_pixel_depth(width: u32, height: u32, pixel: f32) -> Image {
        let mut image = Image::new(width, height);

        let pixel_u32 = pixel.to_bits();
        for p in image.buffer.iter_mut() {
            *p = pixel_u32;
        }

        image
    }

    pub fn from_raw(buffer: Vec<u32>, width: u32, height: u32) -> Option<Image> {
        let w = cast_usize(width);
        let h = cast_usize(height);
        if w * h <= buffer.len() {
            Some(Image {
                width: w,
                height: h,
                buffer,
            })
        } else {
            None
        }
    }

    pub fn into_raw(self) -> Vec<u32> {
        self.buffer
    }

    pub fn pixels_mut_rgba(&mut self) -> PixelsMutRgba<'_> {
        PixelsMutRgba {
            iter: self.buffer.iter_mut(),
        }
    }

    pub fn pixels_mut_depth(&mut self) -> PixelsMutDepth<'_> {
        PixelsMutDepth {
            iter: self.buffer.iter_mut(),
        }
    }

    pub fn pixel_rgba(&self, x: u32, y: u32) -> [u8; 4] {
        let index = cast_usize(y) * self.width + cast_usize(x);
        let pixel_u32 = self.buffer[index];

        pixel_u32.to_le_bytes()
    }

    pub fn pixel_depth(&self, x: u32, y: u32) -> f32 {
        let index = cast_usize(y) * self.width + cast_usize(x);
        let pixel_u32 = self.buffer[index];

        f32::from_bits(pixel_u32)
    }

    pub fn pixel_mut_rgba(&mut self, x: u32, y: u32) -> &mut [u8; 4] {
        let index = cast_usize(y) * self.width + cast_usize(x);
        let pixel_u32 = &mut self.buffer[index];

        unsafe { &mut *(pixel_u32 as *mut u32 as *mut [u8; 4]) }
    }

    pub fn pixel_mut_depth(&mut self, x: u32, y: u32) -> &mut f32 {
        let index = cast_usize(y) * self.width + cast_usize(x);
        let pixel_u32 = &mut self.buffer[index];

        unsafe { &mut *(pixel_u32 as *mut u32 as *mut f32) }
    }

    pub fn sample_nearest_rgba(&self, uv: Vec2) -> Vec4 {
        let u = uv.x.clamp(0.0, 1.0);
        let v = uv.y.clamp(0.0, 1.0);

        let x = u * self.width.saturating_sub(1) as f32;
        let y = v * self.height.saturating_sub(1) as f32;

        let pixel = self.pixel_rgba(x as u32, y as u32);

        Vec4::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
            pixel[3] as f32 / 255.0,
        )
    }

    pub fn set_pixel_rgba(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        *self.pixel_mut_rgba(x, y) = pixel;
    }

    pub fn set_pixel_depth(&mut self, x: u32, y: u32, pixel: f32) {
        *self.pixel_mut_depth(x, y) = pixel;
    }

    pub fn clear_rgba(&mut self, pixel: [u8; 4]) {
        for p in self.pixels_mut_rgba() {
            *p = pixel;
        }
    }

    pub fn clear_depth(&mut self, pixel: f32) {
        for p in self.pixels_mut_depth() {
            *p = pixel;
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        self.height as u32
    }
}

impl AsRef<[u32]> for Image {
    fn as_ref(&self) -> &[u32] {
        &self.buffer
    }
}

pub struct PixelsMutRgba<'a> {
    iter: slice::IterMut<'a, u32>,
}

impl<'a> Iterator for PixelsMutRgba<'a> {
    type Item = &'a mut [u8; 4];

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|v| unsafe { &mut *(v as *mut u32 as *mut [u8; 4]) })
    }
}

pub struct PixelsMutDepth<'a> {
    iter: slice::IterMut<'a, u32>,
}

impl<'a> Iterator for PixelsMutDepth<'a> {
    type Item = &'a mut f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|v| unsafe { &mut *(v as *mut u32 as *mut f32) })
    }
}
