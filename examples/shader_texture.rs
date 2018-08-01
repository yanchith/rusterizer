extern crate failure;
extern crate image;
extern crate nalgebra;
extern crate rusterizer;
extern crate wavefront_obj;

use std::env;
use std::f64;
use std::fs::{self, File};
use std::io::BufReader;

use failure::Error;
use image::{imageops, ImageBuffer, ImageFormat};
use nalgebra::{Vector2, Vector3, Vector4};
use wavefront_obj::obj::{self, ObjSet, Object, Primitive};

use rusterizer::image::{Depth, DepthImage, Rgba, RgbaImage};
use rusterizer::pipeline::Pipeline;
use rusterizer::shader::{ShaderProgram, Smooth};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Attribute {
    pub pos: Vector4<f64>,
    pub norm: Vector3<f64>,
    pub uv: Vector2<f64>,
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
    u_light_dir: Vector3<f64>,
    u_tex: RgbaImage<u8>,
}

impl SimpleProgram {
    pub fn with_light_and_texture(
        light_dir: Vector3<f64>,
        tex: RgbaImage<u8>,
    ) -> SimpleProgram {
        SimpleProgram {
            u_light_dir: light_dir,
            u_tex: tex,
        }
    }
}

impl ShaderProgram for SimpleProgram {
    type Attribute = Attribute;
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

        attr.pos
    }

    fn fragment(
        &self,
        _pos: &Vector4<f64>,
        var: &Self::Varying,
    ) -> Vector4<f64> {
        let tex_color = self.u_tex.sample_nearest::<Vector4<f64>>(&var.uv) / 255.0;
        tex_color * var.light_intensity
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
        DepthImage::from_pixel(WIDTH, HEIGHT, Depth { data: [-1.0] });

    let texture_file = File::open(tex_path)?;
    let texture_reader = BufReader::new(texture_file);
    let texture = imageops::flip_vertical(&image::load(
        texture_reader,
        ImageFormat::TGA,
    )?.to_rgba());

    let model_string = fs::read_to_string(&model_path)?;
    let model = obj::parse(model_string).expect("failed to parse model");

    let attributes = collect_attributes(model);

    let tex_width = texture.width();
    let tex_height = texture.height();
    let program = SimpleProgram::with_light_and_texture(
        Vector3::new(0.0, 0.0, 1.0),
        RgbaImage::from_raw(texture.into_raw(), tex_width, tex_height).unwrap(),
    );
    let pipeline = Pipeline::new(program);

    pipeline.triangles(&attributes, &mut color_image, &mut depth_image);

    let out_depth_image = ImageBuffer::from_fn(WIDTH, HEIGHT, |x, y| {
        let depth = depth_image.pixel(x, y).data[0] * 0.5 + 0.5;
        image::Luma([(depth * 255.0) as u8])
    });

    let out_color_image = ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
        WIDTH,
        HEIGHT,
        color_image.into_raw(),
    ).expect("failed to convert to output image");

    imageops::flip_vertical(&out_color_image)
        .save("./shader_texture-color.png")?;
    imageops::flip_vertical(&out_depth_image)
        .save("./shader_texture-depth.png")?;

    Ok(())
}

fn collect_attributes(objset: ObjSet) -> Vec<Attribute> {
    let mut attrs = Vec::new();
    for object in objset.objects {
        let Object {
            vertices,
            tex_vertices,
            normals,
            geometry,
            ..
        } = object;
        for geom in geometry {
            for shape in geom.shapes {
                match shape.primitive {
                    Primitive::Triangle(idx1, idx2, idx3) => {
                        let v1 = vertices[idx1.0];
                        let v2 = vertices[idx2.0];
                        let v3 = vertices[idx3.0];

                        let vn1 = normals[idx1.2.unwrap()];
                        let vn2 = normals[idx2.2.unwrap()];
                        let vn3 = normals[idx3.2.unwrap()];

                        let vt1 = tex_vertices[idx1.1.unwrap()];
                        let vt2 = tex_vertices[idx2.1.unwrap()];
                        let vt3 = tex_vertices[idx3.1.unwrap()];

                        let world_a = Vector4::new(v1.x, v1.y, v1.z, 1.0);
                        let world_b = Vector4::new(v2.x, v2.y, v2.z, 1.0);
                        let world_c = Vector4::new(v3.x, v3.y, v3.z, 1.0);

                        let norm_a = Vector3::new(vn1.x, vn1.y, vn1.z);
                        let norm_b = Vector3::new(vn2.x, vn2.y, vn2.z);
                        let norm_c = Vector3::new(vn3.x, vn3.y, vn3.z);

                        let tex_a = Vector2::new(vt1.u, vt1.v);
                        let tex_b = Vector2::new(vt2.u, vt2.v);
                        let tex_c = Vector2::new(vt3.u, vt3.v);

                        attrs.push(Attribute {
                            pos: world_a,
                            norm: norm_a,
                            uv: tex_a,
                        });
                        attrs.push(Attribute {
                            pos: world_b,
                            norm: norm_b,
                            uv: tex_b,
                        });
                        attrs.push(Attribute {
                            pos: world_c,
                            norm: norm_c,
                            uv: tex_c,
                        });
                    }
                    _ => { /* NO OP */ }
                }
            }
        }
    }
    attrs
}
