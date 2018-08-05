use std::{f64, mem};

use image::{Depth, DepthImage, Rgba, RgbaImage};
use nalgebra::{U2, Vector2, Vector3, Vector4};

use math;
use shader::{ShaderProgram, Smooth};

pub struct Pipeline<S: ShaderProgram> {
    shader: S,
}

impl<S: ShaderProgram> Pipeline<S> {
    pub fn new(shader: S) -> Self {
        Pipeline { shader }
    }

    pub fn triangles(
        &self,
        // TODO: Consider IntoIterator<Item = S::Attribute> instead of slice
        buffer: &[S::Attribute],
        image_color: &mut RgbaImage<u8>,
        image_depth: &mut DepthImage<f64>,
    ) {
        let width = image_color.width();
        let height = image_color.height();
        let half_width = f64::from(width / 2);
        let half_height = f64::from(height / 2);

        let mut va = S::Varying::default();
        let mut vb = S::Varying::default();
        let mut vc = S::Varying::default();

        for i in 0..buffer.len() / 3 {
            let attr = i * 3;

            let world_a = self.shader.vertex(&buffer[attr], &mut va);
            let world_b = self.shader.vertex(&buffer[attr + 1], &mut vb);
            let world_c = self.shader.vertex(&buffer[attr + 2], &mut vc);

            // TODO: backface culling

            let screen_a = world_to_screen(
                from_homogenous(world_a),
                half_width,
                half_height,
            );
            let screen_b = world_to_screen(
                from_homogenous(world_b),
                half_width,
                half_height,
            );
            let screen_c = world_to_screen(
                from_homogenous(world_c),
                half_width,
                half_height,
            );

            self.triangle(
                image_color,
                image_depth,
                (&screen_a, &screen_b, &screen_c),
                (&va, &vb, &vc),
            );
        }
    }

    pub fn lines(
        &self,
        buffer: &[S::Attribute],
        image_color: &mut RgbaImage<u8>,
    ) {
        let width = image_color.width();
        let height = image_color.height();
        let half_width = f64::from(width / 2);
        let half_height = f64::from(height / 2);

        let mut va = S::Varying::default();
        let mut vb = S::Varying::default();

        for i in 0..buffer.len() / 2 {
            let attr = i * 2;

            let world_a = self.shader.vertex(&buffer[attr], &mut va);
            let world_b = self.shader.vertex(&buffer[attr + 1], &mut vb);

            let screen_a = world_to_screen(world_a, half_width, half_height);
            let screen_b = world_to_screen(world_b, half_width, half_height);

            line(
                image_color,
                // TODO: interpolate color
                Rgba { data: [255; 4] },
                screen_a.x as i32,
                screen_a.y as i32,
                screen_b.x as i32,
                screen_b.y as i32,
            );
        }
    }

    /// Writes a triangle to image and z_buffer.
    fn triangle(
        &self,
        image_color: &mut RgbaImage<u8>,
        image_depth: &mut DepthImage<f64>,
        (a, b, c): (&Vector4<f64>, &Vector4<f64>, &Vector4<f64>),
        (va, vb, vc): (&S::Varying, &S::Varying, &S::Varying),
    ) {
        let width = image_color.width();
        let height = image_color.height();

        // TODO: don't clone, find a way to solve VectorSlice2 type error
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

                    let f_pos = Vector4::interpolate(a, b, c, &bc);
                    let f_var = S::Varying::interpolate(va, vb, vc, &bc);
                    let f_depth = Depth { data: [f_pos.z] };

                    if image_depth.pixel(x, y) < &f_depth {
                        let f_color = self.shader.fragment(&f_pos, &f_var);
                        image_depth.set_pixel(x, y, f_depth);
                        image_color.set_pixel(x, y, vec_to_rgba(f_color));
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
            u32::min(xmax as u32, width.saturating_sub(1)),
            u32::min(ymax as u32, height.saturating_sub(1)),
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

/// Draw a line from src to dst using Bresenham's Algorithm
fn line(
    image: &mut RgbaImage<u8>,
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
            image.set_pixel(y as u32, x as u32, color);
        } else {
            image.set_pixel(x as u32, y as u32, color);
        }

        error2 += derror2;
        if error2 > dx {
            y += if src_y > dst_y { -1 } else { 1 };
            error2 -= dx * 2;
        }
    }
}

fn vec_to_rgba(color: Vector4<f64>) -> Rgba<u8> {
    Rgba {
        data: [
            (math::clamp(color.x, 0.0, 1.0) * 255.0) as u8,
            (math::clamp(color.y, 0.0, 1.0) * 255.0) as u8,
            (math::clamp(color.z, 0.0, 1.0) * 255.0) as u8,
            (math::clamp(color.w, 0.0, 1.0) * 255.0) as u8,
        ],
    }
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

fn from_homogenous(vec: Vector4<f64>) -> Vector4<f64> {
    Vector4::new(vec.x / vec.w, vec.y / vec.w, vec.z / vec.w, 1.0)
}
