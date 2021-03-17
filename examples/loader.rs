use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;

use glam::{Vec2, Vec3, Vec4};
use image::{self, imageops, ImageFormat};
use rusterizer::image::Image;
use wavefront_obj::obj::{self, ObjSet, Object, Primitive};

use crate::attr::Attribute;

pub fn load_image(path: &str) -> Result<Image, Box<dyn Error>> {
    let texture_file = File::open(path)?;
    let texture_reader = BufReader::new(texture_file);
    let texture =
        imageops::flip_vertical(&image::load(texture_reader, ImageFormat::Tga)?.to_rgba8());

    let width = texture.width();
    let height = texture.height();

    let texture_u32 = texture
        .into_raw()
        .chunks(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    Ok(Image::from_raw(texture_u32, width, height).unwrap())
}

pub fn load_model(path: &str) -> Result<Vec<Attribute>, Box<dyn Error>> {
    let model_string = fs::read_to_string(&path)?;
    let model = obj::parse(model_string).expect("failed to parse model");

    Ok(collect_attributes(model))
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

                        let world_a = Vec4::new(v1.x as f32, v1.y as f32, v1.z as f32, 1.0);
                        let world_b = Vec4::new(v2.x as f32, v2.y as f32, v2.z as f32, 1.0);
                        let world_c = Vec4::new(v3.x as f32, v3.y as f32, v3.z as f32, 1.0);

                        let norm_a = Vec3::new(vn1.x as f32, vn1.y as f32, vn1.z as f32);
                        let norm_b = Vec3::new(vn2.x as f32, vn2.y as f32, vn2.z as f32);
                        let norm_c = Vec3::new(vn3.x as f32, vn3.y as f32, vn3.z as f32);

                        let tex_a = Vec2::new(vt1.u as f32, vt1.v as f32);
                        let tex_b = Vec2::new(vt2.u as f32, vt2.v as f32);
                        let tex_c = Vec2::new(vt3.u as f32, vt3.v as f32);

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
