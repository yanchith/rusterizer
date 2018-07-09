extern crate image;
extern crate nalgebra;

mod z_buffer;

use std::mem;
use std::{f64, i32};

use image::{Rgba, RgbaImage};
use nalgebra::{Vector2, Vector3};

pub use z_buffer::ZBuffer;

pub fn line_slow(
    image: &mut RgbaImage,
    color: Rgba<u8>,
    mut src_x: i32,
    mut src_y: i32,
    mut dst_x: i32,
    mut dst_y: i32,
) {
    let transposed = (dst_x - src_x).abs() < (dst_y - src_y).abs();
    if transposed {
        mem::swap(&mut src_x, &mut src_y);
        mem::swap(&mut dst_x, &mut dst_y);
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

pub fn line_fast(
    image: &mut RgbaImage,
    color: Rgba<u8>,
    mut src_x: i32,
    mut src_y: i32,
    mut dst_x: i32,
    mut dst_y: i32,
) {
    let transposed = (dst_x - src_x).abs() < (dst_y - src_y).abs();
    if transposed {
        mem::swap(&mut src_x, &mut src_y);
        mem::swap(&mut dst_x, &mut dst_y);
    }

    if src_x > dst_x {
        mem::swap(&mut src_x, &mut dst_x);
        mem::swap(&mut src_y, &mut dst_y);
    }

    let dx = dst_x - src_x;
    let dy = dst_y - src_y;

    let derror2 = i32::abs(dy) * 2;
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
            y += if src_y > dst_y { -1 } else { 1 };
            error2 -= dx * 2;
        }
    }
}

pub fn triangle(
    image: &mut RgbaImage,
    z_buffer: &mut ZBuffer,
    color: Rgba<u8>,
    a: Vector3<f64>,
    b: Vector3<f64>,
    c: Vector3<f64>,
) {
    fn bounding_box(
        a: Vector3<f64>,
        b: Vector3<f64>,
        c: Vector3<f64>,
        width: u32,
        height: u32,
    ) -> (Vector2<u32>, Vector2<u32>) {
        let xmin = f64::min(f64::min(a.x, b.x), c.x);
        let xmax = f64::max(f64::max(a.x, b.x), c.x);
        let ymin = f64::min(f64::min(a.y, b.y), c.y);
        let ymax = f64::max(f64::max(a.y, b.y), c.y);
        (
            Vector2::new(xmin as u32, ymin as u32),
            Vector2::new(
                u32::min(xmax as u32, width - 1),
                u32::min(ymax as u32, height - 1),
            ),
        )
    }

    fn barycentric(
        a: Vector3<f64>,
        b: Vector3<f64>,
        c: Vector3<f64>,
        p: Vector3<f64>,
    ) -> Vector3<f64> {
        let ab = b - a;
        let ac = c - a;
        let pa = a - p;
        let xs = Vector3::new(ac.x, ab.x, pa.x);
        let ys = Vector3::new(ac.y, ab.y, pa.y);
        let ortho = xs.cross(&ys);
        if f64::abs(ortho.z) < 1.0 {
            Vector3::new(-1.0, -1.0, -1.0)
        } else {
            Vector3::new(
                1.0 - (ortho.x + ortho.y) / ortho.z,
                ortho.y / ortho.z,
                ortho.x / ortho.z,
            )
        }
    }

    let (topleft, bottomright) =
        bounding_box(a, b, c, image.width(), image.height());
    for x in topleft.x..=bottomright.x {
        for y in topleft.y..=bottomright.y {
            let p = Vector3::new(x as f64, y as f64, 0.0);
            let bc = barycentric(a, b, c, p);
            if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                continue;
            }
            let frag_depth = a.z * bc.x + b.z * bc.y + c.z * bc.z;
            if z_buffer.get(x, y) < frag_depth {
                z_buffer.set(x, y, frag_depth);
                image.put_pixel(x, y, color);
            }
        }
    }
}
