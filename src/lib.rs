extern crate image;
extern crate nalgebra;

mod z_buffer;

use std::mem;
use std::{f32, f64, i32};

use image::{Rgba, RgbaImage};
use nalgebra::{Vector2, Vector3};

pub use z_buffer::ZBuffer;

/// Computes a bounding box (in screenspace pixels) for triangle A, B, C.
/// Ignores Z dimensions.
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

/// Computes barycentric coordinates of point P in triangle A, B, C.
/// Returns None for degenerate triangles.
fn barycentric(
    a: Vector3<f64>,
    b: Vector3<f64>,
    c: Vector3<f64>,
    p: Vector3<f64>,
) -> Option<Vector3<f64>> {
    let ab = b - a;
    let ac = c - a;
    let pa = a - p;
    let xs = Vector3::new(ac.x, ab.x, pa.x);
    let ys = Vector3::new(ac.y, ab.y, pa.y);
    let ortho = xs.cross(&ys);
    if f64::abs(ortho.z) < 1.0 {
        None
    } else {
        Some(Vector3::new(
            1.0 - (ortho.x + ortho.y) / ortho.z,
            ortho.y / ortho.z,
            ortho.x / ortho.z,
        ))
    }
}

fn multiply_rgba(color1: Rgba<u8>, color2: Rgba<u8>) -> Rgba<u8> {
    Rgba([
        ((f32::from(color1.data[0]) / 255.0)
            * (f32::from(color2.data[0]) / 255.0)
            * 255.0) as u8,
        ((f32::from(color1.data[1]) / 255.0)
            * (f32::from(color2.data[1]) / 255.0)
            * 255.0) as u8,
        ((f32::from(color1.data[2]) / 255.0)
            * (f32::from(color2.data[2]) / 255.0)
            * 255.0) as u8,
        ((f32::from(color1.data[3]) / 255.0)
            * (f32::from(color2.data[3]) / 255.0)
            * 255.0) as u8,
    ])
}

pub fn line(
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

/// Writes a triangle to image and z_buffer.
pub fn triangle(
    image: &mut RgbaImage,
    z_buffer: &mut ZBuffer,
    light_color: Rgba<u8>,
    a: Vector3<f64>,
    b: Vector3<f64>,
    c: Vector3<f64>,
) {
    let (tl, br) = bounding_box(a, b, c, image.width(), image.height());
    for x in tl.x..=br.x {
        for y in tl.y..=br.y {
            let p = Vector3::new(f64::from(x), f64::from(y), 0.0);
            match barycentric(a, b, c, p) {
                Some(bc) => {
                    if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                        continue;
                    }
                    let frag_depth = a.z * bc.x + b.z * bc.y + c.z * bc.z;
                    if z_buffer.get(x, y) < frag_depth {
                        z_buffer.set(x, y, frag_depth);
                        image.put_pixel(x, y, light_color);
                    }
                }
                None => continue,
            }
        }
    }
}

pub fn triangle_texture(
    image: &mut RgbaImage,
    z_buffer: &mut ZBuffer,
    light_color: Rgba<u8>,
    a: Vector3<f64>,
    b: Vector3<f64>,
    c: Vector3<f64>,
    uva: Vector2<f64>,
    uvb: Vector2<f64>,
    uvc: Vector2<f64>,
    texture: &RgbaImage,
) {
    let (tl, br) = bounding_box(a, b, c, image.width(), image.height());
    for x in tl.x..=br.x {
        for y in tl.y..=br.y {
            let p = Vector3::new(f64::from(x), f64::from(y), 0.0);
            match barycentric(a, b, c, p) {
                Some(bc) => {
                    if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                        continue;
                    }
                    let frag_depth = a.z * bc.x + b.z * bc.y + c.z * bc.z;
                    if z_buffer.get(x, y) < frag_depth {
                        let tc = uva * bc.x + uvb * bc.y + uvc * bc.z;
                        let tx = (f64::from(texture.width() - 1) * tc.x) as u32;
                        let ty =
                            (f64::from(texture.height() - 1) * tc.y) as u32;
                        let tex_color = *texture.get_pixel(tx, ty);

                        z_buffer.set(x, y, frag_depth);
                        image.put_pixel(
                            x,
                            y,
                            multiply_rgba(tex_color, light_color),
                        );
                    }
                }
                None => continue,
            }
        }
    }
}
