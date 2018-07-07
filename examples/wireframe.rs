extern crate failure;
extern crate image;
extern crate rusterizer;
extern crate wavefront_obj;

use std::env;
use std::fs;
use std::i32;

use failure::Error;
use image::{imageops, ImageBuffer, Rgba};
use wavefront_obj::obj::{self, Object, Primitive};

use rusterizer::line_3 as line;

fn black() -> Rgba<u8> {
    Rgba([0, 0, 0, 255])
}

fn white() -> Rgba<u8> {
    Rgba([255, 255, 255, 255])
}

fn main() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let path = args.next().expect("USAGE: prog path");

    let white = white();
    const WIDTH: i32 = 400;
    const HEIGHT: i32 = 400;
    const HALF_WIDTH: f64 = WIDTH as f64 / 2.0;
    const HALF_HEIGHT: f64 = HEIGHT as f64 / 2.0;

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

                        let x1 = i32::min(
                            ((v1.x + 1.0) * HALF_WIDTH) as i32,
                            WIDTH - 1,
                        );
                        let y1 = i32::min(
                            ((v1.y + 1.0) * HALF_HEIGHT) as i32,
                            HEIGHT - 1,
                        );
                        let x2 = i32::min(
                            ((v2.x + 1.0) * HALF_WIDTH) as i32,
                            WIDTH - 1,
                        );
                        let y2 = i32::min(
                            ((v2.y + 1.0) * HALF_HEIGHT) as i32,
                            HEIGHT - 1,
                        );
                        let x3 = i32::min(
                            ((v3.x + 1.0) * HALF_WIDTH) as i32,
                            WIDTH - 1,
                        );
                        let y3 = i32::min(
                            ((v3.y + 1.0) * HALF_HEIGHT) as i32,
                            HEIGHT - 1,
                        );

                        println!(
                            "[{} {}] [{} {}] [{} {}]",
                            x1, x2, y1, y2, x3, y3
                        );

                        line(x1, y1, x2, y2, &mut image, white);
                        line(x2, y2, x3, y3, &mut image, white);
                        line(x3, y3, x1, y1, &mut image, white);
                    }
                    _ => { /* NO OP */ }
                }
            }
        }
    }

    imageops::flip_vertical(&image).save("./wireframe.png")?;

    Ok(())
}
