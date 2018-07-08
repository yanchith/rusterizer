extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate rand;
extern crate rusterizer;
extern crate wavefront_obj;

use std::env;
use std::fs;
use std::i32;

use failure::Error;
use image::{imageops, ImageBuffer, Rgba};
use nalgebra::Vector2;
use rand::prelude::*;
use wavefront_obj::obj::{self, Object, Primitive};

use rusterizer::{line_fast as line, triangle};

fn black() -> Rgba<u8> {
    Rgba([0, 0, 0, 255])
}

fn white() -> Rgba<u8> {
    Rgba([255, 255, 255, 255])
}

fn random_color() -> Rgba<u8> {
    Rgba([random(), random(), random(), 255])
}

fn main() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let path = args.next().expect("USAGE: prog path");

    let white = white();
    const WIDTH: i32 = 800;
    const HEIGHT: i32 = 800;
    const HALF_WIDTH: f64 = WIDTH as f64 * 0.499;
    const HALF_HEIGHT: f64 = HEIGHT as f64 * 0.499;

    let mut image =
        ImageBuffer::from_pixel(WIDTH as u32, HEIGHT as u32, black());

    let model_string = fs::read_to_string(&path)?;
    let model = obj::parse(model_string).expect("failed to parse model");

    for object in model.objects {
        let Object {
            vertices, geometry, ..
        } = object;
        for geom in geometry {
            for shape in geom.shapes {
                match shape.primitive {
                    Primitive::Triangle(idx1, idx2, idx3) => {
                        let v1 = vertices[idx1.0];
                        let v2 = vertices[idx2.0];
                        let v3 = vertices[idx3.0];

                        let x1 = ((v1.x + 1.0) * HALF_WIDTH).floor() as i32;
                        let y1 = ((v1.y + 1.0) * HALF_HEIGHT).floor() as i32;
                        let x2 = ((v2.x + 1.0) * HALF_WIDTH).floor() as i32;
                        let y2 = ((v2.y + 1.0) * HALF_HEIGHT).floor() as i32;
                        let x3 = ((v3.x + 1.0) * HALF_WIDTH).floor() as i32;
                        let y3 = ((v3.y + 1.0) * HALF_HEIGHT).floor() as i32;

                        let a = Vector2::new(x1, y1);
                        let b = Vector2::new(x2, y2);
                        let c = Vector2::new(x3, y3);

                        triangle(&mut image, random_color(), a, b, c);
                    }
                    _ => { /* NO OP */ }
                }
            }
        }
    }

    imageops::flip_vertical(&image).save("./triangle.png")?;

    Ok(())
}
