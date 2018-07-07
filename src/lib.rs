extern crate image;

use std::mem;
use image::{RgbaImage, Rgba};

pub fn line_3(
    mut src_x: i32,
    mut src_y: i32,
    mut dst_x: i32,
    mut dst_y: i32,
    image: &mut RgbaImage,
    color: Rgba<u8>,
) {
    let mut transposed = false;
    if (dst_x - src_x).abs() < (dst_y - src_y).abs() {
        mem::swap(&mut src_x, &mut src_y);
        mem::swap(&mut dst_x, &mut dst_y);
        transposed = true;
    }

    if src_x > dst_x {
        mem::swap(&mut src_x, &mut dst_x);
        mem::swap(&mut src_y, &mut dst_y);
    }

    for x in src_x..=dst_x {
        let t = (x - src_x) as f32 / (dst_x - src_x) as f32;
        let y = src_y as f32 * (1.0 - t) + dst_y as f32 * t;
        if transposed {
            image.put_pixel(y as u32, x as u32, color);
        } else {
            image.put_pixel(x as u32, y as u32, color);
        }
    }
}

pub fn line_4(
    mut src_x: i32,
    mut src_y: i32,
    mut dst_x: i32,
    mut dst_y: i32,
    image: &mut RgbaImage,
    color: Rgba<u8>,
) {
    let mut transposed = false;
    if (dst_x - src_x).abs() < (dst_y - src_y).abs() {
        mem::swap(&mut src_x, &mut src_y);
        mem::swap(&mut dst_x, &mut dst_y);
        transposed = true;
    }

    if src_x > dst_x {
        mem::swap(&mut src_x, &mut dst_x);
        mem::swap(&mut src_y, &mut dst_y);
    }

    let dx = dst_x - src_x;
    let dy = dst_y - src_y;

    let derror = dy as f32 / dx as f32;
    let mut error = 0.0;

    let mut y = src_y;
    for x in src_x..=dst_x {
        if transposed {
            image.put_pixel(y as u32, x as u32, color);
        } else {
            image.put_pixel(x as u32, y as u32, color);
        }

        error += derror;
        if error > 0.5 {
            y += if src_x > dst_x { -1 } else { 1 };
            error -= 1.0;
        }
    }
}

pub fn line_5(
    mut src_x: i32,
    mut src_y: i32,
    mut dst_x: i32,
    mut dst_y: i32,
    image: &mut RgbaImage,
    color: Rgba<u8>,
) {
    let mut transposed = false;
    if (dst_x - src_x).abs() < (dst_y - src_y).abs() {
        mem::swap(&mut src_x, &mut src_y);
        mem::swap(&mut dst_x, &mut dst_y);
        transposed = true;
    }

    if src_x > dst_x {
        mem::swap(&mut src_x, &mut dst_x);
        mem::swap(&mut src_y, &mut dst_y);
    }

    let dx = dst_x - src_x;
    let dy = dst_y - src_y;

    let derror2 = dy * 2;
    let mut error2 = 0;

    let mut y = src_y;
    for x in src_x..=dst_x {
        if transposed {
            image.put_pixel(y as u32, x as u32, color);
        } else {
            image.put_pixel(x as u32, y as u32, color);
        }

        error2 += derror2;
        if error2 > dx {
            y += if src_x > dst_x { -1 } else { 1 };
            error2 -= dx * 2;
        }
    }
}
