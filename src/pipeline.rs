use std::mem;
use std::{f32, f64, i32};

use image::{Rgba, RgbaImage};
use nalgebra::{U3, Vector2, Vector3, Vector4};

use shader::Shader;
use z_buffer::ZBuffer;

pub struct Pipeline<S> {
    shader: S,
}

impl<S: Shader> Pipeline<S> {
    pub fn new(shader: S) -> Self {
        Pipeline { shader }
    }

    pub fn run(
        &self,
        viewport: [u32; 2],
        buffer: &[S::Attribute],
        framebuffer: &mut RgbaImage,
        z_buffer: &mut ZBuffer,
    ) {
        let len_div_3 = buffer.len() / 3;
        for i in 0..len_div_3 {
            let attr = i * 3;
            let a = self.shader
                .vertex(&buffer[attr], &mut S::Varying::default());
            let b = self.shader
                .vertex(&buffer[attr + 1], &mut S::Varying::default());
            let c = self.shader
                .vertex(&buffer[attr + 2], &mut S::Varying::default());
            self.triangle(framebuffer, z_buffer, a, b, c);
        }
    }

    /// Writes a triangle to image and z_buffer.
    fn triangle(
        &self,
        image_color: &mut RgbaImage,
        image_depth: &mut ZBuffer,
        a: Vector4<f64>,
        b: Vector4<f64>,
        c: Vector4<f64>,
    ) {
        let (tl, br) =
            bounding_box(a, b, c, image_color.width(), image_color.height());
        for x in tl.x..=br.x {
            for y in tl.y..=br.y {
                let p = Vector3::new(f64::from(x), f64::from(y), 0.0);
                match barycentric(
                    a.fixed_rows::<U3>(0).clone_owned(),
                    b.fixed_rows::<U3>(0).clone_owned(),
                    c.fixed_rows::<U3>(0).clone_owned(),
                    p,
                ) {
                    Some(bc) => {
                        if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                            continue;
                        }
                        let frag_pos = Vector3::new(
                            a.x * bc.x + b.x * bc.y + c.x * bc.z,
                            a.y * bc.x + b.y * bc.y + c.y * bc.z,
                            a.z * bc.x + b.z * bc.y + c.z * bc.z,
                        );
                        if image_depth.get(x, y) < frag_pos.z {
                            let mut frag_color = Rgba([0, 0, 0, 0]);
                            self.shader.fragment(
                                &frag_pos,
                                &S::Varying::default(),
                                &mut frag_color,
                            );
                            image_depth.set(x, y, frag_pos.z);

                            image_color.put_pixel(x, y, frag_color);
                        }
                    }
                    None => continue,
                }
            }
        }
    }
}

/// Computes a bounding box (in screenspace pixels) for triangle A, B, C.
/// Ignores Z dimensions.
fn bounding_box(
    a: Vector4<f64>,
    b: Vector4<f64>,
    c: Vector4<f64>,
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
