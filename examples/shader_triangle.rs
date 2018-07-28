extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate rusterizer;

use std::f64;

use failure::Error;
use image::{imageops, ImageBuffer};
use nalgebra::{Vector3, Vector4};

use rusterizer::image::{Depth, DepthImage, Rgba, RgbaImage};
use rusterizer::pipeline::Pipeline;
use rusterizer::shader::{ShaderProgram, Smooth};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn black() -> Rgba<u8> {
    Rgba {
        data: [0, 0, 0, 255],
    }
}

fn depth() -> Depth<f64> {
    Depth { data: [-1.0] }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Attribute {
    pub pos: Vector4<f64>,
    pub clr: Vector4<f64>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Varying {
    pub clr: Vector4<f64>,
}

impl Default for Varying {
    fn default() -> Varying {
        Varying {
            clr: Vector4::zeros(),
        }
    }
}

impl Smooth for Varying {
    fn interpolate(
        a: Varying,
        b: Varying,
        c: Varying,
        bc: Vector3<f64>,
    ) -> Varying {
        Varying {
            clr: Vector4::interpolate(a.clr, b.clr, c.clr, bc),
        }
    }
}

struct SimpleProgram;

impl ShaderProgram for SimpleProgram {
    type Attribute = Attribute;
    type Varying = Varying;

    fn vertex(
        &self,
        attr: &Self::Attribute,
        var: &mut Self::Varying,
    ) -> Vector4<f64> {
        var.clr = attr.clr;
        attr.pos
    }

    fn fragment(
        &self,
        _pos: &Vector4<f64>,
        var: &Self::Varying,
    ) -> Vector4<f64> {
        var.clr
    }
}

fn main() -> Result<(), Error> {
    let mut color_image = RgbaImage::from_pixel(WIDTH, HEIGHT, black());
    let mut depth_image = DepthImage::from_pixel(WIDTH, HEIGHT, depth());

    let attributes = vec![
        Attribute {
            pos: Vector4::new(-1.0, 0.0, 0.0, 1.0),
            clr: Vector4::new(1.0, 0.0, 0.0, 1.0),
        },
        Attribute {
            pos: Vector4::new(0.0, 1.0, 0.0, 1.0),
            clr: Vector4::new(0.0, 1.0, 0.0, 1.0),
        },
        Attribute {
            pos: Vector4::new(1.0, -1.0, 0.0, 1.0),
            clr: Vector4::new(0.0, 0.0, 1.0, 1.0),
        },
    ];

    let pipeline = Pipeline::new(SimpleProgram);

    pipeline.triangles(&attributes, &mut color_image, &mut depth_image);

    let out_color_image = ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
        WIDTH,
        HEIGHT,
        color_image.into_raw(),
    ).expect("failed to convert to output image");

    imageops::flip_vertical(&out_color_image).save("./shader_triangle.png")?;

    Ok(())
}
