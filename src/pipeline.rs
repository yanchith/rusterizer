use std::f64;

use image::{Rgba, RgbaImage};
use nalgebra::{U3, Vector2, Vector3, Vector4};

use shader::{Shader, Smooth};
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
        buffer: &[S::Attribute],
        framebuffer: &mut RgbaImage,
        z_buffer: &mut ZBuffer,
    ) {
        let width = framebuffer.width();
        let height = framebuffer.height();
        let half_width = (width / 2) as f64;
        let half_height = (height / 2) as f64;

        let len_div_3 = buffer.len() / 3;
        for i in 0..len_div_3 {
            let attr = i * 3;
            let mut vara = S::Varying::default();
            let mut varb = S::Varying::default();
            let mut varc = S::Varying::default();
            let world_a = self.shader.vertex(&buffer[attr], &mut vara);
            let world_b = self.shader.vertex(&buffer[attr + 1], &mut varb);
            let world_c = self.shader.vertex(&buffer[attr + 2], &mut varc);
            let screen_a = world_to_screen(world_a, half_width, half_height);
            let screen_b = world_to_screen(world_b, half_width, half_height);
            let screen_c = world_to_screen(world_c, half_width, half_height);
            self.triangle(
                framebuffer,
                z_buffer,
                screen_a,
                screen_b,
                screen_c,
                vara,
                varb,
                varc,
            );
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
        vara: S::Varying,
        varb: S::Varying,
        varc: S::Varying,
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
                    Some(bar) => {
                        if bar.x < 0.0 || bar.y < 0.0 || bar.z < 0.0 {
                            continue;
                        }
                        let frag_pos = Vector4::interpolate(a, b, c, bar);
                        let var =
                            S::Varying::interpolate(vara, varb, varc, bar);
                        if image_depth.get(x, y) < frag_pos.z {
                            let mut frag_color =
                                Vector4::new(0.0, 0.0, 0.0, 0.0);
                            let discard = self.shader.fragment(
                                &frag_pos,
                                &var,
                                &mut frag_color,
                            );
                            if !discard {
                                image_depth.set(x, y, frag_pos.z);
                                image_color.put_pixel(
                                    x,
                                    y,
                                    color_vec_to_rgba(frag_color),
                                );
                            }
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

fn color_vec_to_rgba(color: Vector4<f64>) -> Rgba<u8> {
    let remapped = color * 255.0;
    Rgba([
        remapped.x as u8,
        remapped.y as u8,
        remapped.z as u8,
        remapped.w as u8,
    ])
}

fn world_to_screen(
    world_coords: Vector4<f64>,
    half_width: f64,
    half_height: f64,
) -> Vector4<f64> {
    Vector4::new(
        (world_coords.x + 1.0) * half_width,
        (world_coords.y + 1.0) * half_height,
        world_coords.z,
        world_coords.w,
    )
}
