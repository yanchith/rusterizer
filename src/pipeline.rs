use std::f64;

use image::{Rgba, RgbaImage};
use nalgebra::{U2, Vector2, Vector3, Vector4};

use shader::{Shader, Smooth};
use z_buffer::ZBuffer;

pub struct Pipeline<S: Shader> {
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
        let half_width = f64::from(width / 2);
        let half_height = f64::from(height / 2);

        let len_div_3 = buffer.len() / 3;
        for i in 0..len_div_3 {
            let attr = i * 3;

            let mut va = S::Varying::default();
            let mut vb = S::Varying::default();
            let mut vc = S::Varying::default();

            let world_a = self.shader.vertex(&buffer[attr], &mut va);
            let world_b = self.shader.vertex(&buffer[attr + 1], &mut vb);
            let world_c = self.shader.vertex(&buffer[attr + 2], &mut vc);

            let screen_a = world_to_screen(world_a, half_width, half_height);
            let screen_b = world_to_screen(world_b, half_width, half_height);
            let screen_c = world_to_screen(world_c, half_width, half_height);

            self.triangle(
                framebuffer,
                z_buffer,
                (screen_a, screen_b, screen_c),
                (va, vb, vc),
            );
        }
    }

    /// Writes a triangle to image and z_buffer.
    fn triangle(
        &self,
        image_color: &mut RgbaImage,
        image_depth: &mut ZBuffer,
        (a, b, c): (Vector4<f64>, Vector4<f64>, Vector4<f64>),
        (va, vb, vc): (S::Varying, S::Varying, S::Varying),
    ) {
        let width = image_color.width();
        let height = image_color.height();

        // TODO: don't clone, use VectorSlice2
        let a2 = a.fixed_rows::<U2>(0).clone_owned();
        let b2 = b.fixed_rows::<U2>(0).clone_owned();
        let c2 = c.fixed_rows::<U2>(0).clone_owned();

        let (topleft, bottomright) = bounding_box(a2, b2, c2, width, height);

        for x in topleft.x..=bottomright.x {
            for y in topleft.y..=bottomright.y {
                let point = Vector2::new(f64::from(x), f64::from(y));
                if let Some(bc) = barycentric(a2, b2, c2, point) {
                    if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                        continue;
                    }

                    let f_pos = Vector4::interpolate(a, b, c, bc);
                    let f_var = S::Varying::interpolate(va, vb, vc, bc);

                    if image_depth.get(x, y) < f_pos.z {
                        let f_color = self.shader.fragment(&f_pos, &f_var);
                        image_depth.set(x, y, f_pos.z);
                        image_color.put_pixel(x, y, color_vec_to_rgba(f_color));
                    }
                }
            }
        }
    }
}

/// Computes a bounding box (in screenspace coords) for triangle A, B, C.
fn bounding_box(
    a: Vector2<f64>,
    b: Vector2<f64>,
    c: Vector2<f64>,
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
    a: Vector2<f64>,
    b: Vector2<f64>,
    c: Vector2<f64>,
    p: Vector2<f64>,
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
