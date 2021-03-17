use glam::{Vec2, Vec3, Vec4};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Attribute {
    pub pos: Vec4,
    pub norm: Vec3,
    pub uv: Vec2,
}
