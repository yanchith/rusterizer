use image::Rgba;
use nalgebra::{Vector3, Vector4};

pub trait Smooth: Copy + Clone {
    fn interpolate(a: Self, b: Self, c: Self, bar: Vector3<f64>) -> Self;
}

pub trait Shader {
    type Attribute: Default;
    type Varying: Default + Smooth;
    // type Fragment: Default;

    fn vertex(
        &self,
        attr: &Self::Attribute,
        var: &mut Self::Varying,
    ) -> Vector4<f64>;

    fn fragment(
        &self,
        pos: &Vector3<f64>,
        var: &Self::Varying,
        frag: &mut Rgba<u8>, // &mut Self::Fragment
    ) -> bool;
}

impl Smooth for f32 {
    fn interpolate(a: f32, b: f32, c: f32, bar: Vector3<f64>) -> f32 {
        (f64::from(a) * bar.x + f64::from(b) * bar.y + f64::from(c) * bar.z)
            as f32
    }
}

impl Smooth for f64 {
    fn interpolate(a: f64, b: f64, c: f64, bar: Vector3<f64>) -> f64 {
        a * bar.x + b * bar.y + c * bar.z
    }
}
