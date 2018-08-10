use std::fmt::Debug;
use std::slice::{Chunks, ChunksMut};

use nalgebra::{Vector1, Vector2, Vector4};
use num::{Zero, Num};

use math;

pub type RgbaImage<T> = Image<Rgba<T>>;
pub type DepthImage<T> = Image<Depth<T>>;

pub trait ColorData: Num + Copy + Clone + Debug {}

impl ColorData for usize {}
impl ColorData for u8 {}
impl ColorData for u16 {}
impl ColorData for u32 {}
impl ColorData for u64 {}
impl ColorData for u128 {}

impl ColorData for isize {}
impl ColorData for i8 {}
impl ColorData for i16 {}
impl ColorData for i32 {}
impl ColorData for i64 {}
impl ColorData for i128 {}

impl ColorData for f32 {}
impl ColorData for f64 {}

pub trait Pixel {
    type ColorChannel: ColorData;
    fn channel_count() -> u8;
    fn from_slice(slice: &[Self::ColorChannel]) -> &Self;
    fn from_slice_mut(slice: &mut [Self::ColorChannel]) -> &mut Self;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Image<P: Pixel> {
    width: usize,
    height: usize,
    buffer: Vec<P::ColorChannel>,
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
        T: ColorData,
        P: Pixel<ColorChannel = T>,
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

    pub fn into_raw(self) -> Vec<P::ColorChannel> {
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

    // TODO: make uv generic over all float vectors
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

    pub fn clear(&mut self, pixel: P) {
        for p in self.pixels_mut() {
            *p = pixel;
        }
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
    P::ColorChannel: 'a,
{
    chunks: Chunks<'a, P::ColorChannel>,
}

impl<'a, P> Iterator for Pixels<'a, P>
where
    P: Pixel + 'a,
    P::ColorChannel: 'a,
{
    type Item = &'a P;

    fn next(&mut self) -> Option<&'a P> {
        self.chunks.next().map(|v| P::from_slice(v))
    }
}

pub struct PixelsMut<'a, P>
where
    P: Pixel + 'a,
    P::ColorChannel: 'a,
{
    chunks: ChunksMut<'a, P::ColorChannel>,
}

impl<'a, P> Iterator for PixelsMut<'a, P>
where
    P: Pixel + 'a,
    P::ColorChannel: 'a,
{
    type Item = &'a mut P;

    fn next(&mut self) -> Option<&'a mut P> {
        self.chunks.next().map(|v| P::from_slice_mut(v))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rgba<T: Num> {
    pub data: [T; 4],
}

impl<T: ColorData> Pixel for Rgba<T> {
    type ColorChannel = T;

    fn channel_count() -> u8 {
        4
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

impl<T: ColorData + 'static> From<Vector4<T>> for Rgba<T> {
    fn from(v: Vector4<T>) -> Rgba<T> {
        Rgba {
            data: [v.x, v.y, v.z, v.w],
        }
    }
}

// Orphan rules T_T
impl<T, U> Into<Vector4<T>> for Rgba<U>
where
    T: ColorData + 'static,
    U: Into<T> + ColorData + 'static,
{
    fn into(self) -> Vector4<T> {
        let [r, g, b, a] = self.data;
        Vector4::new(r.into(), g.into(), b.into(), a.into())
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Depth<T: ColorData> {
    pub data: [T; 1],
}

impl<T: ColorData> Pixel for Depth<T> {
    type ColorChannel = T;

    fn channel_count() -> u8 {
        1
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

impl<T: ColorData + 'static> From<Vector1<T>> for Depth<T> {
    fn from(depth: Vector1<T>) -> Depth<T> {
        Depth {
            data: [depth.x],
        }
    }
}

// Orphan rules T_T
impl<T, U> Into<Vector1<T>> for Depth<U>
where
    T: ColorData + 'static,
    U: Into<T> + ColorData + 'static,
{
    fn into(self) -> Vector1<T> {
        let [depth] = self.data;
        Vector1::new(depth.into())
    }
}
