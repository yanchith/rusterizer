extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate rusterizer;

use std::f64;

use failure::Error;
use image::{imageops, ImageBuffer, Rgba};
use nalgebra::Vector4;

use rusterizer::pipeline::Pipeline;
use rusterizer::shader::Shader;
use rusterizer::ZBuffer;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

fn black() -> Rgba<u8> {
    Rgba([0, 0, 0, 255])
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Attribute {
    pos: Vector4<f64>,
}

struct SimpleShader;

impl Shader for SimpleShader {
    type Attribute = Attribute;
    type Varying = ();

    fn vertex(
        &self,
        attr: &Self::Attribute,
        _var: &mut Self::Varying,
    ) -> Vector4<f64> {
        attr.pos
    }

    fn fragment(
        &self,
        _pos: &Vector4<f64>,
        _var: &Self::Varying,
        frag: &mut Vector4<f64>,
    ) -> bool {
        *frag = Vector4::new(1.0, 0.0, 0.0, 1.0);
        false
    }
}

fn main() -> Result<(), Error> {
    let mut image = ImageBuffer::from_pixel(WIDTH, HEIGHT, black());
    let mut z_buffer = ZBuffer::new(WIDTH, HEIGHT, -1.0);

    let attributes = vec![
        Attribute {
            pos: Vector4::new(-1.0, 0.0, 0.0, 1.0),
        },
        Attribute {
            pos: Vector4::new(0.0, 1.0, 0.0, 1.0),
        },
        Attribute {
            pos: Vector4::new(1.0, -1.0, 0.0, 1.0),
        },
    ];

    let pipeline = Pipeline::new(SimpleShader);

    pipeline.run(&attributes, &mut image, &mut z_buffer);

    imageops::flip_vertical(&image).save("./shader-triangle.png")?;

    Ok(())
}
