use std::fmt::Debug;
use std::slice::{Chunks, ChunksMut};

use nalgebra::{Vector2, Vector4};
use num::traits::{Bounded, Num, NumCast};
use num::Zero;

use math;

/// Wishlist for image (most of functionality can be inspired by
/// image::ImageBuffer) - serves both as color and depth buffer
/// - generic over image channel count: RED, RG, RGB, RGBA
/// - generic over data type: all primitives
/// - get and set data as Vector{1-4} depending on channel count
/// - get by pixel index (texelFetch())
///   * (u32, u32)
///   * Vector2<u32>
/// - get by uv index  (texture())
///   * (f64, f64)
///   * Vector2<f64>
/// - interop
///   * into_raw
///   * from_raw

pub type RgbaImage<T> = Image<Rgba<T>>;
pub type DepthImage<T> = Image<Depth<T>>;

pub trait Pixel {
    type DataType: Primitive;
    fn channel_count() -> u8;
    fn as_slice(&self) -> &[Self::DataType];
    fn as_slice_mut(&mut self) -> &mut [Self::DataType];
    fn from_slice(slice: &[Self::DataType]) -> &Self;
    fn from_slice_mut(slice: &mut [Self::DataType]) -> &mut Self;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Image<P: Pixel> {
    width: usize,
    height: usize,
    buffer: Vec<P::DataType>,
}

impl<P: Pixel + Copy> Image<P> {
    pub fn new(width: u32, height: u32) -> Image<P> {
        let w = width as usize;
        let h = height as usize;
        let c = P::channel_count() as usize;
        Image {
            width: w,
            height: h,
            buffer: vec![Zero::zero(); w * h * c],
        }
    }

    pub fn from_pixel(width: u32, height: u32, pixel: P) -> Image<P> {
        let mut image = Image::new(width, height);
        for p in image.pixels_mut() {
            *p = pixel;
        }
        image
    }

    pub fn from_raw<T>(
        buffer: Vec<T>,
        width: u32,
        height: u32,
    ) -> Option<Image<P>>
    where
        T: Primitive,
        P: Pixel<DataType = T>,
    {
        let w = width as usize;
        let h = height as usize;
        let c = P::channel_count() as usize;
        if w * h * c <= buffer.len() {
            Some(Image {
                width: w,
                height: h,
                buffer,
            })
        } else {
            None
        }
    }

    pub fn into_raw(self) -> Vec<P::DataType> {
        self.buffer
    }

    pub fn pixels(&self) -> Pixels<P> {
        let channel_count = P::channel_count() as usize;
        Pixels {
            chunks: self.buffer.chunks(channel_count),
        }
    }

    pub fn pixels_mut(&mut self) -> PixelsMut<P> {
        let channel_count = P::channel_count() as usize;
        PixelsMut {
            chunks: self.buffer.chunks_mut(channel_count),
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> &P {
        let channel_count = P::channel_count() as usize;
        let index = channel_count * (y as usize * self.width + x as usize);
        P::from_slice(&self.buffer[index..index + channel_count])
    }

    pub fn pixel_mut(&mut self, x: u32, y: u32) -> &mut P {
        let channel_count = P::channel_count() as usize;
        let index = channel_count * (y as usize * self.width + x as usize);
        P::from_slice_mut(&mut self.buffer[index..index + channel_count])
    }

    // TODO: make generic over all float vectors
    pub fn sample_nearest<V>(&self, uv: &Vector2<f64>) -> V
    where
        P: Into<V>,
    {
        let u = math::clamp(uv.x, 0.0, 1.0);
        let v = math::clamp(uv.y, 0.0, 1.0);

        let x = u * self.width.saturating_sub(1) as f64;
        let y = v * self.height.saturating_sub(1) as f64;

        let pixel = self.pixel(x as u32, y as u32);
        Into::<V>::into(*pixel)
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: P) {
        *self.pixel_mut(x, y) = pixel;
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        self.height as u32
    }
}

pub struct Pixels<'a, P>
where
    P: Pixel + 'a,
    P::DataType: 'a,
{
    chunks: Chunks<'a, P::DataType>,
}

impl<'a, P> Iterator for Pixels<'a, P>
where
    P: Pixel + 'a,
    P::DataType: 'a,
{
    type Item = &'a P;

    fn next(&mut self) -> Option<&'a P> {
        self.chunks.next().map(|v| P::from_slice(v))
    }
}

pub struct PixelsMut<'a, P>
where
    P: Pixel + 'a,
    P::DataType: 'a,
{
    chunks: ChunksMut<'a, P::DataType>,
}

impl<'a, P> Iterator for PixelsMut<'a, P>
where
    P: Pixel + 'a,
    P::DataType: 'a,
{
    type Item = &'a mut P;

    fn next(&mut self) -> Option<&'a mut P> {
        self.chunks.next().map(|v| P::from_slice_mut(v))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rgba<T: Primitive> {
    pub data: [T; 4],
}

impl<T: Primitive> Pixel for Rgba<T> {
    type DataType = T;
    fn channel_count() -> u8 {
        4
    }

    fn as_slice(&self) -> &[T] {
        &self.data
    }

    fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    fn from_slice(slice: &[T]) -> &Self {
        assert_eq!(slice.len(), 4);
        unsafe { &*(slice.as_ptr() as *const Rgba<T>) }
    }

    fn from_slice_mut(slice: &mut [T]) -> &mut Self {
        assert_eq!(slice.len(), 4);
        unsafe { &mut *(slice.as_mut_ptr() as *mut Rgba<T>) }
    }
}

impl<T: Primitive + 'static> From<Vector4<T>> for Rgba<T> {
    fn from(vector4: Vector4<T>) -> Rgba<T> {
        Rgba {
            data: [vector4.x, vector4.y, vector4.z, vector4.w],
        }
    }
}

// Orphan rules T_T
// impl<T: Primitive + 'static> From<Rgba<T>> for Vector4<T> {
//     fn from(rgba: Rgba<T>) -> Vector4<T> {
//         let [r, g, b, a] = rgba.data;
//         Vector4::new(r, g, b, a)
//     }
// }

impl<T: Primitive + 'static> Into<Vector4<T>> for Rgba<T> {
    fn into(self) -> Vector4<T> {
        let [r, g, b, a] = self.data;
        Vector4::new(r, g, b, a)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Depth<T: Primitive> {
    pub data: [T; 1],
}

impl<T: Primitive> Pixel for Depth<T> {
    type DataType = T;

    fn channel_count() -> u8 {
        1
    }

    fn as_slice(&self) -> &[T] {
        &self.data
    }

    fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    fn from_slice(slice: &[T]) -> &Self {
        assert_eq!(slice.len(), 1);
        unsafe { &*(slice.as_ptr() as *const Depth<T>) }
    }

    fn from_slice_mut(slice: &mut [T]) -> &mut Self {
        assert_eq!(slice.len(), 1);
        unsafe { &mut *(slice.as_mut_ptr() as *mut Depth<T>) }
    }
}

pub trait Primitive:
    Debug + Clone + Copy + PartialOrd<Self> + Num + NumCast + Bounded
{
    // Empty
}

impl Primitive for usize {}
impl Primitive for u8 {}
impl Primitive for u16 {}
impl Primitive for u32 {}
impl Primitive for u64 {}
impl Primitive for u128 {}

impl Primitive for isize {}
impl Primitive for i8 {}
impl Primitive for i16 {}
impl Primitive for i32 {}
impl Primitive for i64 {}
impl Primitive for i128 {}

impl Primitive for f32 {}
impl Primitive for f64 {}
