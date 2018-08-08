extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate rusterizer;
extern crate wavefront_obj;

use std::env;
use std::f64;
use std::f64::consts::PI;

use failure::Error;
use nalgebra::{Matrix4, Point3, Vector2, Vector3, Vector4};
use rusterizer::image::{Depth, DepthImage, Rgba, RgbaImage};
use rusterizer::pipeline::Pipeline;
use rusterizer::shader::{ShaderProgram, Smooth};

mod attr;
mod loader;

const WIDTH: u32 = 80;
const HEIGHT: u32 = 80;

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
    fn interpolate(
        a: &Varying,
        b: &Varying,
        c: &Varying,
        bc: &Vector3<f64>,
    ) -> Varying {
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
}

impl ShaderProgram for SimpleProgram {
    type Attribute = attr::Attribute;
    type Varying = Varying;

    fn vertex(
        &self,
        attr: &Self::Attribute,
        var: &mut Self::Varying,
    ) -> Vector4<f64> {
        let normal = nalgebra::normalize(&attr.norm);
        let light_intensity = nalgebra::dot(&normal, &self.u_light_dir);

        var.norm = normal;
        var.uv = attr.uv;
        var.light_intensity = light_intensity;

        self.u_proj * self.u_view * attr.pos
    }

    fn fragment(
        &self,
        _pos: &Vector4<f64>,
        var: &Self::Varying,
    ) -> Vector4<f64> {
        let color_tex =
            self.u_tex.sample_nearest::<Vector4<f64>>(&var.uv) / 255.0;
        let color = color_tex * var.light_intensity;
        Vector4::new(color.x, color.y, color.z, 1.0)
    }
}

fn main() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let model_path = args.next().expect("USAGE: prog modelpath texpath");
    let tex_path = args.next().expect("USAGE: prog modelpath texpath");

    let mut color_image = RgbaImage::from_pixel(
        WIDTH,
        HEIGHT,
        Rgba {
            data: [0, 0, 0, 255],
        },
    );
    let mut depth_image =
        DepthImage::from_pixel(WIDTH, HEIGHT, Depth { data: [1.0] });

    let texture = loader::load_image(&tex_path)?;
    let attributes = loader::load_model(&model_path)?;

    let proj = Matrix4::new_perspective(
        WIDTH as f64 / HEIGHT as f64,
        PI / 4.0,
        0.1,
        10.0,
    );

    let view = Matrix4::look_at_rh(
        &Point3::new(0.0, 0.0, 3.0),
        &Point3::new(0.0, 0.0, 0.0),
        &Vector3::new(0.0, 1.0, 0.0),
    );

    let pipeline = Pipeline::new(SimpleProgram::with_uniforms(
        proj,
        view,
        Vector3::new(0.0, 0.0, 1.0),
        texture,
    ));

    pipeline.triangles(&attributes, &mut color_image, &mut depth_image);

    let output = render(&color_image);

    // Hide cursor while printing canvas to avoid flickering
    print!("\x1B[?25l{}\x1B[?25h\n", output);

    // Move cursor up to enable drawing of next frame over the current one
    print!("\x1B[{}A", HEIGHT / 2);

    Ok(())
}

/// Returns a string that, when printed to the terminal, renders the given image.
fn render(image: &RgbaImage<u8>) -> String {
    // The image should not be empty and must have an even number of rows because
    // two rows are represented by each line of output
    assert!(image.height() > 0 && image.width() > 0);
    assert!(image.height() % 2 == 0);

    let mut output = String::new();

    let row_length = image.width();
    let row_count = image.height() / 2;

    for i in 0..row_count {
        for j in 0..row_length {
            let top = image.pixel(j, 2 * i);
            let bottom = image.pixel(j, 2 * i + 1);
            // Unicode UPPER HALF BLOCK with foreground (top) and background
            // (bottom) color
            let [tr, tg, tb, _] = top.data;
            let [br, bg, bb, _] = bottom.data;
            let block = format!(
                "\x1B[38;2;{};{};{};48;2;{};{};{}m\u{2580}",
                tr, tg, tb, br, bg, bb,
            );

            output.push_str(&block);
        }

        let last_line = i == row_count - 1;

        // Always reset on the last line to restore foreground color
        if last_line {
            output.push_str("\x1B[m");
        } else {
            output.push_str("\n");
        }
    }

    output
}
