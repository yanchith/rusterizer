use nalgebra::{Vector2, Vector3, Vector4};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attribute {
    pub pos: Vector4<f64>,
    pub norm: Vector3<f64>,
    pub uv: Vector2<f64>,
}
