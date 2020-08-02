use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;

use image::{self, imageops, ImageFormat};
use nalgebra::{Vector2, Vector3, Vector4};
use rusterizer::image::RgbaImage;
use wavefront_obj::obj::{self, ObjSet, Object, Primitive};

use crate::attr::Attribute;

pub fn load_image(path: &str) -> Result<RgbaImage<u8>, Box<dyn Error>> {
    let texture_file = File::open(path)?;
    let texture_reader = BufReader::new(texture_file);
    let texture =
        imageops::flip_vertical(&image::load(texture_reader, ImageFormat::Tga)?.to_rgba());

    let width = texture.width();
    let height = texture.height();

    Ok(RgbaImage::from_raw(texture.into_raw(), width, height).unwrap())
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
