use nalgebra::{Vector2, Vector3, Vector4};

pub trait Smooth: Copy + Clone {
    fn interpolate(a: Self, b: Self, c: Self, bar: Vector3<f64>) -> Self;
}

pub trait Shader {
    type Attribute;
    type Varying: Default + Smooth;
    // type Fragment: Default;

    fn vertex(
        &self,
        attr: &Self::Attribute,
        var: &mut Self::Varying,
    ) -> Vector4<f64>;

    fn fragment(
        &self,
        pos: &Vector4<f64>,
        var: &Self::Varying,
        frag: &mut Vector4<f64>, // &mut Self::Fragment
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

impl Smooth for Vector2<f32> {
    fn interpolate(
        a: Vector2<f32>,
        b: Vector2<f32>,
        c: Vector2<f32>,
        bar: Vector3<f64>,
    ) -> Vector2<f32> {
        Vector2::new(
            f32::interpolate(a.x, b.x, c.x, bar),
            f32::interpolate(a.y, b.y, c.y, bar),
        )
    }
}

impl Smooth for Vector3<f32> {
    fn interpolate(
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
        bar: Vector3<f64>,
    ) -> Vector3<f32> {
        Vector3::new(
            f32::interpolate(a.x, b.x, c.x, bar),
            f32::interpolate(a.y, b.y, c.y, bar),
            f32::interpolate(a.z, b.z, c.z, bar),
        )
    }
}

impl Smooth for Vector4<f32> {
    fn interpolate(
        a: Vector4<f32>,
        b: Vector4<f32>,
        c: Vector4<f32>,
        bar: Vector3<f64>,
    ) -> Vector4<f32> {
        Vector4::new(
            f32::interpolate(a.x, b.x, c.x, bar),
            f32::interpolate(a.y, b.y, c.y, bar),
            f32::interpolate(a.z, b.z, c.z, bar),
            f32::interpolate(a.w, b.w, c.w, bar),
        )
    }
}

impl Smooth for Vector2<f64> {
    fn interpolate(
        a: Vector2<f64>,
        b: Vector2<f64>,
        c: Vector2<f64>,
        bar: Vector3<f64>,
    ) -> Vector2<f64> {
        Vector2::new(
            f64::interpolate(a.x, b.x, c.x, bar),
            f64::interpolate(a.y, b.y, c.y, bar),
        )
    }
}

impl Smooth for Vector3<f64> {
    fn interpolate(
        a: Vector3<f64>,
        b: Vector3<f64>,
        c: Vector3<f64>,
        bar: Vector3<f64>,
    ) -> Vector3<f64> {
        Vector3::new(
            f64::interpolate(a.x, b.x, c.x, bar),
            f64::interpolate(a.y, b.y, c.y, bar),
            f64::interpolate(a.z, b.z, c.z, bar),
        )
    }
}

impl Smooth for Vector4<f64> {
    fn interpolate(
        a: Vector4<f64>,
        b: Vector4<f64>,
        c: Vector4<f64>,
        bar: Vector3<f64>,
    ) -> Vector4<f64> {
        Vector4::new(
            f64::interpolate(a.x, b.x, c.x, bar),
            f64::interpolate(a.y, b.y, c.y, bar),
            f64::interpolate(a.z, b.z, c.z, bar),
            f64::interpolate(a.w, b.w, c.w, bar),
        )
    }
}

impl Smooth for () {
    fn interpolate(_: (), _: (), _: (), _: Vector3<f64>) -> () {}
}
