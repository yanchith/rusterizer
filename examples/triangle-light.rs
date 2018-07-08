extern crate image;
extern crate nalgebra;
extern crate wavefront_obj;
extern crate failure;
extern crate rusterizer;

use std::env;
use std::fs;
use std::i32;

use image::{imageops, ImageBuffer, Rgba};
use nalgebra::{Vector2, Vector3};
use wavefront_obj::obj::{self, Object, Primitive};
use failure::Error;

use rusterizer::triangle;

fn black() -> Rgba<u8> {
    Rgba([0, 0, 0, 255])
}

fn luma(intensity: f64) -> Rgba<u8> {
    let intense = (255.0 * intensity) as u8;
    Rgba([intense, intense, intense, 255])
}

fn main() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let path = args.next().expect("USAGE: prog path");

    let light_dir = Vector3::new(0.0, 0.0, -1.0);

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

                        let world_a = Vector3::new(v1.x, v1.y, v1.z);
                        let world_b = Vector3::new(v2.x, v2.y, v2.z);
                        let world_c = Vector3::new(v3.x, v3.y, v3.z);

                        let normal =
                            (world_c - world_a).cross(&(world_b - world_a));
                        let norm_normal = nalgebra::normalize(&normal);
                        let light_intensity =
                            nalgebra::dot(&norm_normal, &light_dir);

                        if light_intensity > 0.0 {
                            let screen_a = Vector2::new(x1, y1);
                            let screen_b = Vector2::new(x2, y2);
                            let screen_c = Vector2::new(x3, y3);

                            triangle(
                                &mut image,
                                luma(light_intensity),
                                screen_a,
                                screen_b,
                                screen_c,
                            );
                        }
                    }
                    _ => { /* NO OP */ }
                }
            }
        }
    }

    imageops::flip_vertical(&image).save("./triangle-light.png")?;

    Ok(())
}
