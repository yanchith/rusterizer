use std::env;
use std::error::Error;
use std::f32;
use std::thread;
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3, Vec4};
use minifb::{Window, WindowOptions};
use rusterizer::image::Image;
use rusterizer::shader::{ShaderProgram, Smooth};
use rusterizer::{CullFace, Pipeline, PipelineOptions};

// TODO(yan): Rustfmt doesn't like these paths in 1.50.0
#[rustfmt::skip]
#[path = "../attr.rs"]
mod attr;
#[rustfmt::skip]
#[path = "../loader.rs"]
mod loader;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn black() -> [u8; 4] {
    [0, 0, 0, 255]
}

fn depth() -> f32 {
    1.0
}
#[derive(Debug, PartialEq, Clone, Copy)]
struct Varying {
    pub norm: Vec3,
    pub uv: Vec2,
    pub light_intensity: f32,
}

impl Default for Varying {
    fn default() -> Varying {
        Varying {
            norm: Vec3::ZERO,
            uv: Vec2::ZERO,
            light_intensity: 0.0,
        }
    }
}

impl Smooth for Varying {
    fn interpolate(a: &Varying, b: &Varying, c: &Varying, bc: Vec3) -> Varying {
        Varying {
            norm: Vec3::interpolate(&a.norm, &b.norm, &c.norm, bc),
            uv: Vec2::interpolate(&a.uv, &b.uv, &c.uv, bc),
            light_intensity: f32::interpolate(
                &a.light_intensity,
                &b.light_intensity,
                &c.light_intensity,
                bc,
            ),
        }
    }
}

struct SimpleProgram {
    u_proj: Mat4,
    u_view: Mat4,
    u_light_dir: Vec3,
    u_tex: Image,
}

impl SimpleProgram {
    pub fn with_uniforms(proj: Mat4, view: Mat4, light_dir: Vec3, tex: Image) -> SimpleProgram {
        SimpleProgram {
            u_proj: proj,
            u_view: view,
            u_light_dir: light_dir,
            u_tex: tex,
        }
    }

    pub fn set_view(&mut self, view: Mat4) {
        self.u_view = view;
    }
}

impl ShaderProgram for SimpleProgram {
    type Attribute = attr::Attribute;
    type Varying = Varying;

    fn vertex(&self, attr: &Self::Attribute, var: &mut Self::Varying) -> Vec4 {
        let normal = attr.norm.normalize();
        let light_intensity = normal.dot(self.u_light_dir);

        var.norm = normal;
        var.uv = attr.uv;
        var.light_intensity = light_intensity;

        self.u_proj * self.u_view * attr.pos
    }

    fn fragment(&self, _pos: Vec4, var: &Self::Varying) -> Vec4 {
        let color_tex = self.u_tex.sample_nearest_rgba(var.uv);
        let color = color_tex * var.light_intensity;

        Vec4::new(color.x, color.y, color.z, 1.0)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let model_path = args.next().expect("USAGE: prog modelpath texpath");
    let tex_path = args.next().expect("USAGE: prog modelpath texpath");

    let mut color_image = Image::from_pixel_rgba(WIDTH, HEIGHT, black());
    let mut depth_image = Image::from_pixel_depth(WIDTH, HEIGHT, depth());

    let texture = loader::load_image(&tex_path)?;
    let attributes = loader::load_model(&model_path)?;

    let proj = Mat4::perspective_rh_gl(
        WIDTH as f32 / HEIGHT as f32,
        f32::consts::PI / 4.0,
        0.1,
        10.0,
    );

    let view = Mat4::look_at_rh(
        Vec3::new(0.0, 0.0, 3.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut shader = SimpleProgram::with_uniforms(proj, view, Vec3::new(0.0, 0.0, 1.0), texture);

    let mut window_image = Vec::with_capacity(WIDTH as usize * HEIGHT as usize);
    let mut window = Window::new(
        "Rusterizer",
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions::default(),
    )
    .unwrap();

    let pipeline = Pipeline::with_options(PipelineOptions {
        cull_face: CullFace::Back,
        ..PipelineOptions::default()
    });

    let start_time = Instant::now();
    let frame_duration = Duration::from_millis(33);

    loop {
        let total_duration = start_time.elapsed();
        let frame_start_time = Instant::now();

        let t = total_duration.as_secs_f32();
        let view = Mat4::look_at_rh(
            Vec3::new(3.0 * t.sin(), 0.0, 3.0 * t.cos()),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );

        shader.set_view(view);

        color_image.clear_rgba(black());
        depth_image.clear_depth(depth());
        pipeline.triangles(&shader, &attributes, &mut color_image, &mut depth_image);

        // minifb buffer expects BGRA, our image is RGBA; do some shuffling
        let pixel_iter = color_image.as_ref().iter().map(|pixel| {
            let mut color = 0u32;

            let [r, g, b, a] = pixel.to_le_bytes();

            color |= u32::from(b);
            color |= u32::from(g) << 8;
            color |= u32::from(r) << 16;
            color |= u32::from(a) << 24;

            color
        });

        window_image.clear();
        window_image.extend(pixel_iter);
        window
            .update_with_buffer(&window_image, WIDTH as usize, HEIGHT as usize)
            .unwrap();

        let draw_duration = frame_start_time.elapsed();
        println!("frame time: {:?}", draw_duration);

        // Try to sleep for the remainder of the frame
        let sleep_duration = frame_duration.checked_sub(draw_duration);

        // If some, duration is not negative
        if let Some(duration) = sleep_duration {
            thread::sleep(duration);
        }
    }
}
