use glam::{Vec2, Vec3, Vec4};

pub trait Smooth {
    fn interpolate(a: &Self, b: &Self, c: &Self, bc: Vec3) -> Self;
}

pub trait ShaderProgram {
    type Attribute;
    type Varying: Default + Smooth;
    // type Fragment;

    fn vertex(&self, attribute: &Self::Attribute, varying: &mut Self::Varying) -> Vec4;

    fn fragment(&self, position: Vec4, varying: &Self::Varying) -> Vec4;
}

impl Smooth for f32 {
    fn interpolate(a: &f32, b: &f32, c: &f32, bc: Vec3) -> f32 {
        a * bc.x + b * bc.y + c * bc.z
    }
}

impl Smooth for Vec2 {
    fn interpolate(a: &Vec2, b: &Vec2, c: &Vec2, bc: Vec3) -> Vec2 {
        Vec2::new(
            f32::interpolate(&a.x, &b.x, &c.x, bc),
            f32::interpolate(&a.y, &b.y, &c.y, bc),
        )
    }
}

impl Smooth for Vec3 {
    fn interpolate(a: &Vec3, b: &Vec3, c: &Vec3, bc: Vec3) -> Vec3 {
        Vec3::new(
            f32::interpolate(&a.x, &b.x, &c.x, bc),
            f32::interpolate(&a.y, &b.y, &c.y, bc),
            f32::interpolate(&a.z, &b.z, &c.z, bc),
        )
    }
}

impl Smooth for Vec4 {
    fn interpolate(a: &Vec4, b: &Vec4, c: &Vec4, bc: Vec3) -> Vec4 {
        Vec4::new(
            f32::interpolate(&a.x, &b.x, &c.x, bc),
            f32::interpolate(&a.y, &b.y, &c.y, bc),
            f32::interpolate(&a.z, &b.z, &c.z, bc),
            f32::interpolate(&a.w, &b.w, &c.w, bc),
        )
    }
}
