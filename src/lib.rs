pub mod image;
pub mod shader;

mod convert;

use glam::{Vec2, Vec3, Vec4};

use crate::image::Image;
use crate::shader::{ShaderProgram, Smooth};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CullFace {
    None,
    Front,
    Back,
    FrontAndBack,
}

impl Default for CullFace {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct PipelineOptions {
    pub cull_face: CullFace,
}

pub struct Pipeline {
    options: PipelineOptions,
}

impl Pipeline {
    pub fn with_options(options: PipelineOptions) -> Pipeline {
        Pipeline { options }
    }

    pub fn triangles<S: ShaderProgram>(
        &self,
        shader: &S,
        buffer: &[S::Attribute],
        image_color: &mut Image,
        image_depth: &mut Image,
    ) {
        let width = image_color.width();
        let height = image_color.height();

        assert!(width == image_depth.width(), "images must have equal dims");
        assert!(
            height == image_depth.height(),
            "images must have equal dims"
        );

        let half_width = width as f32 / 2.0;
        let half_height = height as f32 / 2.0;

        for i in 0..buffer.len() / 3 {
            let attr = i * 3;

            let mut var_a = S::Varying::default();
            let mut var_b = S::Varying::default();
            let mut var_c = S::Varying::default();

            let world_a = shader.vertex(&buffer[attr], &mut var_a);
            let world_b = shader.vertex(&buffer[attr + 1], &mut var_b);
            let world_c = shader.vertex(&buffer[attr + 2], &mut var_c);

            if self.options.cull_face != CullFace::None {
                let normal = face_normal(
                    Vec3::new(world_a.x, world_a.y, world_a.z),
                    Vec3::new(world_b.x, world_b.y, world_b.z),
                    Vec3::new(world_c.x, world_c.y, world_c.z),
                );

                let do_cull = match self.options.cull_face {
                    CullFace::FrontAndBack => true,
                    CullFace::Front => normal.z > 0.0,
                    CullFace::Back => normal.z < 0.0,
                    CullFace::None => unreachable!(),
                };

                if do_cull {
                    continue;
                }
            }

            // TODO: clipping
            // TODO: viewport transform

            let screen_a = world_to_screen(from_homogenous(world_a), half_width, half_height);
            let screen_b = world_to_screen(from_homogenous(world_b), half_width, half_height);
            let screen_c = world_to_screen(from_homogenous(world_c), half_width, half_height);

            self.triangle(
                shader,
                image_color,
                image_depth,
                (screen_a, screen_b, screen_c),
                (&var_a, &var_b, &var_c),
            );
        }
    }

    /// Writes a triangle to image and z_buffer.
    fn triangle<S: ShaderProgram>(
        &self,
        shader: &S,
        image_color: &mut Image,
        image_depth: &mut Image,
        (a, b, c): (Vec4, Vec4, Vec4),
        (va, vb, vc): (&S::Varying, &S::Varying, &S::Varying),
    ) {
        let width = image_color.width();
        let height = image_color.height();

        let a2 = Vec2::new(a.x, a.y);
        let b2 = Vec2::new(b.x, b.y);
        let c2 = Vec2::new(c.x, c.y);

        let (minx, miny, maxx, maxy) = bounding_box(a2, b2, c2, width, height);

        for x in minx..=maxx {
            for y in miny..=maxy {
                let point = Vec2::new(x as f32, y as f32);
                if let Some(bc) = barycentric(a2, b2, c2, point) {
                    if bc.x < 0.0 || bc.y < 0.0 || bc.z < 0.0 {
                        continue;
                    }

                    // Compute frag depth and remap it from NDC to [0..1]
                    let mut f_pos = Vec4::interpolate(&a, &b, &c, bc);
                    f_pos.z = f_pos.z / 2.0 + 0.5;
                    let f_depth = f_pos.z;

                    // GL_LESS
                    let flipped_y = height - 1 - y;
                    if f_depth < image_depth.pixel_depth(x, flipped_y) {
                        let f_var = S::Varying::interpolate(va, vb, vc, bc);
                        let f_color = shader.fragment(f_pos, &f_var);

                        image_depth.set_pixel_depth(x, flipped_y, f_depth);
                        image_color.set_pixel_rgba(x, flipped_y, vec_to_rgba(f_color));
                    }
                }
            }
        }
    }
}

/// Compute a normal vector for the face A, B, C
fn face_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    ab.cross(ac)
}

/// Compute a bounding box (in screenspace coords) for triangle A, B, C.
fn bounding_box(a: Vec2, b: Vec2, c: Vec2, width: u32, height: u32) -> (u32, u32, u32, u32) {
    let xmin = f32::min(f32::min(a.x, b.x), c.x);
    let xmax = f32::max(f32::max(a.x, b.x), c.x);
    let ymin = f32::min(f32::min(a.y, b.y), c.y);
    let ymax = f32::max(f32::max(a.y, b.y), c.y);
    (
        xmin as u32,
        ymin as u32,
        (xmax as u32).min(width.saturating_sub(1)),
        (ymax as u32).min(height.saturating_sub(1)),
    )
}

/// Computes barycentric coordinates of point P in triangle A, B, C. Returns
/// None for degenerate triangles.
fn barycentric(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> Option<Vec3> {
    let ab = b - a;
    let ac = c - a;
    let pa = a - p;
    let xs = Vec3::new(ac.x, ab.x, pa.x);
    let ys = Vec3::new(ac.y, ab.y, pa.y);
    let ortho = xs.cross(ys);
    if f32::abs(ortho.z) < 1.0 {
        None
    } else {
        Some(Vec3::new(
            1.0 - (ortho.x + ortho.y) / ortho.z,
            ortho.y / ortho.z,
            ortho.x / ortho.z,
        ))
    }
}

fn vec_to_rgba(color: Vec4) -> [u8; 4] {
    [
        (color.x.clamp(0.0, 1.0) * 255.0) as u8,
        (color.y.clamp(0.0, 1.0) * 255.0) as u8,
        (color.z.clamp(0.0, 1.0) * 255.0) as u8,
        (color.w.clamp(0.0, 1.0) * 255.0) as u8,
    ]
}

fn world_to_screen(world_coords: Vec4, half_width: f32, half_height: f32) -> Vec4 {
    Vec4::new(
        (world_coords.x + 1.0) * half_width,
        (world_coords.y + 1.0) * half_height,
        world_coords.z.clamp(-1.0, 1.0),
        world_coords.w,
    )
}

fn from_homogenous(vec: Vec4) -> Vec4 {
    Vec4::new(vec.x / vec.w, vec.y / vec.w, vec.z / vec.w, 1.0 / vec.w)
}

// pub fn lines(
//     &self,
//     buffer: &[S::Attribute],
//     image_color: &mut RgbaImage<u8>,
// ) {
//     let width = image_color.width();
//     let height = image_color.height();
//     let half_width = f64::from(width / 2);
//     let half_height = f64::from(height / 2);

//     let mut va = S::Varying::default();
//     let mut vb = S::Varying::default();

//     for i in 0..buffer.len() / 2 {
//         let attr = i * 2;

//         let world_a = self.shader.vertex(&buffer[attr], &mut va);
//         let world_b = self.shader.vertex(&buffer[attr + 1], &mut vb);

//         let screen_a = world_to_screen(world_a, half_width, half_height);
//         let screen_b = world_to_screen(world_b, half_width, half_height);

//         line(
//             image_color,
//             // TODO: interpolate color
//             Rgba { data: [255; 4] },
//             screen_a.x as i32,
//             screen_a.y as i32,
//             screen_b.x as i32,
//             screen_b.y as i32,
//         );
//     }
// }

// /// Draw a line from src to dst using Bresenham's Algorithm
// fn line(
//     image: &mut RgbaImage<u8>,
//     color: Rgba<u8>,
//     mut src_x: i32,
//     mut src_y: i32,
//     mut dst_x: i32,
//     mut dst_y: i32,
// ) {
//     let transposed = (dst_x - src_x).abs() < (dst_y - src_y).abs();
//     if transposed {
//         mem::swap(&mut src_x, &mut src_y);
//         mem::swap(&mut dst_x, &mut dst_y);
//     }

//     if src_x > dst_x {
//         mem::swap(&mut src_x, &mut dst_x);
//         mem::swap(&mut src_y, &mut dst_y);
//     }

//     let dx = dst_x - src_x;
//     let dy = dst_y - src_y;

//     let derror2 = i32::abs(dy) * 2;
//     let mut error2 = 0;

//     let mut y = src_y;
//     for x in src_x..=dst_x {
//         if transposed {
//             image.set_pixel(y as u32, x as u32, color);
//         } else {
//             image.set_pixel(x as u32, y as u32, color);
//         }

//         error2 += derror2;
//         if error2 > dx {
//             y += if src_y > dst_y { -1 } else { 1 };
//             error2 -= dx * 2;
//         }
//     }
// }
