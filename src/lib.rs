extern crate image;
extern crate nalgebra;

use nalgebra::{Vector2, Vector3};

use std::mem;
use std::{f32, i32};

use image::{Rgba, RgbaImage};

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
    color: Rgba<u8>,
    a: Vector2<i32>,
    b: Vector2<i32>,
    c: Vector2<i32>,
) {
    fn bounding_box(
        a: Vector2<i32>,
        b: Vector2<i32>,
        c: Vector2<i32>,
    ) -> (Vector2<i32>, Vector2<i32>) {
        let xmin = i32::min(i32::min(a.x, b.x), c.x);
        let xmax = i32::max(i32::max(a.x, b.x), c.x);
        let ymin = i32::min(i32::min(a.y, b.y), c.y);
        let ymax = i32::max(i32::max(a.y, b.y), c.y);
        (Vector2::new(xmin, ymin), Vector2::new(xmax, ymax))
    }

    fn barycentric(
        a: Vector2<i32>,
        b: Vector2<i32>,
        c: Vector2<i32>,
        p: Vector2<i32>,
    ) -> Vector3<f32> {
        let ab = b - a;
        let ac = c - a;
        let pa = a - p;
        let xs = Vector3::new(ac.x as f32, ab.x as f32, pa.x as f32);
        let ys = Vector3::new(ac.y as f32, ab.y as f32, pa.y as f32);
        let ortho = xs.cross(&ys);
        if f32::abs(ortho.z) < 1.0 {
            Vector3::new(-1.0, -1.0, -1.0)
        } else {
            Vector3::new(
                1.0 - (ortho.x + ortho.y) / ortho.z,
                ortho.y / ortho.z,
                ortho.x / ortho.z,
            )
        }
    }

    let (topleft, bottomright) = bounding_box(a, b, c);
    for x in topleft.x..=bottomright.x {
        for y in topleft.y..=bottomright.y {
            let p = Vector2::new(x, y);
            let bc = barycentric(a, b, c, p);
            if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                continue;
            }
            image.put_pixel(p.x as u32, p.y as u32, color);
        }
    }
}
