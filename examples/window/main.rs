use std::env;
use std::error::Error;
use std::f64::consts::PI;
use std::thread;
use std::time::{Duration, Instant};

use minifb::{Window, WindowOptions};
use nalgebra::{Matrix4, Point3, Vector2, Vector3, Vector4};
use rusterizer::image::{Depth, DepthImage, Rgba, RgbaImage};
use rusterizer::shader::{ShaderProgram, Smooth};
use rusterizer::{CullFace, Pipeline, PipelineOptions};

mod attr;
mod loader;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

fn black() -> Rgba<u8> {
    Rgba {
        data: [0, 0, 0, 255],
    }
}

fn depth() -> Depth<f64> {
    Depth { data: [1.0] }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Varying {
    pub norm: Vector3<f64>,
    pub uv: Vector2<f64>,
    pub light_intensity: f64,
}

impl Default for Varying {
    fn default() -> Varying {
        Varying {
            norm: Vector3::zeros(),
            uv: Vector2::zeros(),
            light_intensity: 0.0,
        }
    }
}

impl Smooth for Varying {
    fn interpolate(a: &Varying, b: &Varying, c: &Varying, bc: &Vector3<f64>) -> Varying {
        Varying {
            norm: Vector3::interpolate(&a.norm, &b.norm, &c.norm, bc),
            uv: Vector2::interpolate(&a.uv, &b.uv, &c.uv, bc),
            light_intensity: f64::interpolate(
                &a.light_intensity,
                &b.light_intensity,
                &c.light_intensity,
                bc,
            ),
        }
    }
}

struct SimpleProgram {
    u_proj: Matrix4<f64>,
    u_view: Matrix4<f64>,
    u_light_dir: Vector3<f64>,
    u_tex: RgbaImage<u8>,
}

impl SimpleProgram {
    pub fn with_uniforms(
        proj: Matrix4<f64>,
        view: Matrix4<f64>,
        light_dir: Vector3<f64>,
        tex: RgbaImage<u8>,
    ) -> SimpleProgram {
        SimpleProgram {
            u_proj: proj,
            u_view: view,
            u_light_dir: light_dir,
            u_tex: tex,
        }
    }

    pub fn set_view(&mut self, view: Matrix4<f64>) {
        self.u_view = view;
    }
}

impl ShaderProgram for SimpleProgram {
    type Attribute = attr::Attribute;
    type Varying = Varying;

    fn vertex(&self, attr: &Self::Attribute, var: &mut Self::Varying) -> Vector4<f64> {
        let normal = attr.norm.normalize();
        let light_intensity = normal.dot(&self.u_light_dir);

        var.norm = normal;
        var.uv = attr.uv;
        var.light_intensity = light_intensity;

        self.u_proj * self.u_view * attr.pos
    }

    fn fragment(&self, _pos: &Vector4<f64>, var: &Self::Varying) -> Vector4<f64> {
        let color_tex = self.u_tex.sample_nearest::<Vector4<f64>>(&var.uv) / 255.0;
        let color = color_tex * var.light_intensity;
        Vector4::new(color.x, color.y, color.z, 1.0)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let model_path = args.next().expect("USAGE: prog modelpath texpath");
    let tex_path = args.next().expect("USAGE: prog modelpath texpath");

    let mut color_image = RgbaImage::from_pixel(WIDTH, HEIGHT, black());
    let mut depth_image = DepthImage::from_pixel(WIDTH, HEIGHT, depth());

    let texture = loader::load_image(&tex_path)?;
    let attributes = loader::load_model(&model_path)?;

    let proj = Matrix4::new_perspective(WIDTH as f64 / HEIGHT as f64, PI / 4.0, 0.1, 10.0);

    let view = Matrix4::look_at_rh(
        &Point3::new(0.0, 0.0, 3.0),
        &Point3::new(0.0, 0.0, 0.0),
        &Vector3::new(0.0, 1.0, 0.0),
    );

    let mut shader = SimpleProgram::with_uniforms(proj, view, Vector3::new(0.0, 0.0, 1.0), texture);

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

        let t = total_duration.as_secs_f64();
        let view = Matrix4::look_at_rh(
            &Point3::new(3.0 * t.sin(), 0.0, 3.0 * t.cos()),
            &Point3::new(0.0, 0.0, 0.0),
            &Vector3::new(0.0, 1.0, 0.0),
        );

        shader.set_view(view);

        color_image.clear(black());
        depth_image.clear(depth());
        pipeline.triangles(&shader, &attributes, &mut color_image, &mut depth_image);

        // minifb buffer expects BGRA, our image is RGBA; do some shuffling
        let pixel_iter = color_image.as_ref().chunks(4).map(|chunk| {
            let mut color = 0u32;
            // B
            color |= u32::from(chunk[2]);
            // G
            color |= u32::from(chunk[1]) << 8;
            // R
            color |= u32::from(chunk[0]) << 16;
            // A
            color |= u32::from(chunk[3]) << 24;
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
