extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate rusterizer;

use std::f64;

use failure::Error;
use image::{imageops, ImageBuffer, Rgba};
use nalgebra::{Vector3, Vector4};

use rusterizer::pipeline::Pipeline;
use rusterizer::shader::{ShaderProgram, Smooth};
use rusterizer::ZBuffer;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn black() -> Rgba<u8> {
    Rgba([0, 0, 0, 255])
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
            clr: Vector4::<f64>::interpolate(a.clr, b.clr, c.clr, bc),
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
    let mut image = ImageBuffer::from_pixel(WIDTH, HEIGHT, black());
    let mut z_buffer = ZBuffer::new(WIDTH, HEIGHT, -1.0);

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

    pipeline.triangles(&attributes, &mut image, &mut z_buffer);

    imageops::flip_vertical(&image).save("./shader_triangle.png")?;

    Ok(())
}
