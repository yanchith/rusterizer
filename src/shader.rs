use nalgebra::{Vector2, Vector3, Vector4};

pub trait Smooth: Copy + Clone {
    fn interpolate(a: Self, b: Self, c: Self, bc: Vector3<f64>) -> Self;
}

pub trait ShaderProgram {
    type Attribute;
    type Varying: Default + Smooth;
    // type Fragment: Default;

    fn vertex(
        &self,
        attribute: &Self::Attribute,
        varying: &mut Self::Varying,
    ) -> Vector4<f64>;

    fn fragment(
        &self,
        position: &Vector4<f64>,
        varying: &Self::Varying,
    ) -> Vector4<f64>;
}

impl Smooth for f32 {
    fn interpolate(a: f32, b: f32, c: f32, bc: Vector3<f64>) -> f32 {
        (f64::from(a) * bc.x + f64::from(b) * bc.y + f64::from(c) * bc.z) as f32
    }
}

impl Smooth for f64 {
    fn interpolate(a: f64, b: f64, c: f64, bc: Vector3<f64>) -> f64 {
        a * bc.x + b * bc.y + c * bc.z
    }
}

impl Smooth for Vector2<f32> {
    fn interpolate(
        a: Vector2<f32>,
        b: Vector2<f32>,
        c: Vector2<f32>,
        bc: Vector3<f64>,
    ) -> Vector2<f32> {
        Vector2::new(
            f32::interpolate(a.x, b.x, c.x, bc),
            f32::interpolate(a.y, b.y, c.y, bc),
        )
    }
}

impl Smooth for Vector3<f32> {
    fn interpolate(
        a: Vector3<f32>,
        b: Vector3<f32>,
        c: Vector3<f32>,
        bc: Vector3<f64>,
    ) -> Vector3<f32> {
        Vector3::new(
            f32::interpolate(a.x, b.x, c.x, bc),
            f32::interpolate(a.y, b.y, c.y, bc),
            f32::interpolate(a.z, b.z, c.z, bc),
        )
    }
}

impl Smooth for Vector4<f32> {
    fn interpolate(
        a: Vector4<f32>,
        b: Vector4<f32>,
        c: Vector4<f32>,
        bc: Vector3<f64>,
    ) -> Vector4<f32> {
        Vector4::new(
            f32::interpolate(a.x, b.x, c.x, bc),
            f32::interpolate(a.y, b.y, c.y, bc),
            f32::interpolate(a.z, b.z, c.z, bc),
            f32::interpolate(a.w, b.w, c.w, bc),
        )
    }
}

impl Smooth for Vector2<f64> {
    fn interpolate(
        a: Vector2<f64>,
        b: Vector2<f64>,
        c: Vector2<f64>,
        bc: Vector3<f64>,
    ) -> Vector2<f64> {
        Vector2::new(
            f64::interpolate(a.x, b.x, c.x, bc),
            f64::interpolate(a.y, b.y, c.y, bc),
        )
    }
}

impl Smooth for Vector3<f64> {
    fn interpolate(
        a: Vector3<f64>,
        b: Vector3<f64>,
        c: Vector3<f64>,
        bc: Vector3<f64>,
    ) -> Vector3<f64> {
        Vector3::new(
            f64::interpolate(a.x, b.x, c.x, bc),
            f64::interpolate(a.y, b.y, c.y, bc),
            f64::interpolate(a.z, b.z, c.z, bc),
        )
    }
}

impl Smooth for Vector4<f64> {
    fn interpolate(
        a: Vector4<f64>,
        b: Vector4<f64>,
        c: Vector4<f64>,
        bc: Vector3<f64>,
    ) -> Vector4<f64> {
        Vector4::new(
            f64::interpolate(a.x, b.x, c.x, bc),
            f64::interpolate(a.y, b.y, c.y, bc),
            f64::interpolate(a.z, b.z, c.z, bc),
            f64::interpolate(a.w, b.w, c.w, bc),
        )
    }
}

impl Smooth for () {
    fn interpolate(_: (), _: (), _: (), _: Vector3<f64>) -> () {}
}
