extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate rusterizer;
extern crate wavefront_obj;

use std::env;
use std::fs;
use std::{f64, i32};

use failure::Error;
use image::{imageops, ImageBuffer, Luma, Rgba};
use nalgebra::Vector3;
use wavefront_obj::obj::{self, Object, Primitive};

use rusterizer::{triangle, ZBuffer};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 800;
const HALF_WIDTH: f64 = WIDTH as f64 * 0.5;
const HALF_HEIGHT: f64 = HEIGHT as f64 * 0.5;

fn black() -> Rgba<u8> {
    Rgba([0, 0, 0, 255])
}

fn luma(intensity: f64) -> Rgba<u8> {
    let intense = (255.0 * intensity) as u8;
    Rgba([intense, intense, intense, 255])
}

fn world_to_screen(
    world_coords: Vector3<f64>,
    half_width: f64,
    half_height: f64,
) -> Vector3<f64> {
    Vector3::new(
        (world_coords.x + 1.0) * half_width,
        (world_coords.y + 1.0) * half_height,
        world_coords.z,
    )
}

fn main() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let path = args.next().expect("USAGE: prog path");

    let light_dir = Vector3::new(0.0, 0.0, -1.0);

    let mut image =
        ImageBuffer::from_pixel(WIDTH as u32, HEIGHT as u32, black());
    let mut z_buffer = ZBuffer::new(WIDTH as u32, HEIGHT as u32, -1.0);

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

                        let world_a = Vector3::new(v1.x, v1.y, v1.z);
                        let world_b = Vector3::new(v2.x, v2.y, v2.z);
                        let world_c = Vector3::new(v3.x, v3.y, v3.z);

                        let normal =
                            (world_c - world_a).cross(&(world_b - world_a));
                        let norm_normal = nalgebra::normalize(&normal);
                        let light_intensity =
                            nalgebra::dot(&norm_normal, &light_dir);

                        if light_intensity > 0.0 {
                            let screen_a = world_to_screen(
                                world_a,
                                HALF_WIDTH,
                                HALF_HEIGHT,
                            );
                            let screen_b = world_to_screen(
                                world_b,
                                HALF_WIDTH,
                                HALF_HEIGHT,
                            );
                            let screen_c = world_to_screen(
                                world_c,
                                HALF_WIDTH,
                                HALF_HEIGHT,
                            );

                            triangle(
                                &mut image,
                                &mut z_buffer,
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

    let z_buffer_image =
        ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x, y| {
            let depth = z_buffer.get(x, y) * 0.5 + 0.5;
            Luma([(depth * 255.0) as u8])
        });

    imageops::flip_vertical(&image).save("./triangle_light_per_tri.png")?;
    imageops::flip_vertical(&z_buffer_image)
        .save("./triangle_light_per_tri-depth.png")?;

    Ok(())
}
