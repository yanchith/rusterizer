extern crate image;
extern crate failure;
extern crate rusterizer;

use failure::Error;
use image::{ImageBuffer, Rgba};

use rusterizer::{line_slow, line_fast};

fn black() -> Rgba<u8> {
    Rgba([0, 0, 0, 255])
}

fn red() -> Rgba<u8> {
    Rgba([255, 0, 0, 255])
}

fn main() -> Result<(), Error> {
    let mut image_slow = ImageBuffer::from_pixel(6, 6, black());
    let mut image_fast = ImageBuffer::from_pixel(6, 6, black());

    line_slow(&mut image_slow, red(), 2, 1, 5, 5);
    line_slow(&mut image_slow, red(), 5, 5, 1, 4);
    line_slow(&mut image_slow, red(), 1, 4, 2, 1);

    line_fast(&mut image_fast, red(), 2, 1, 5, 5);
    line_fast(&mut image_fast, red(), 5, 5, 1, 4);
    line_fast(&mut image_fast, red(), 1, 4, 2, 1);

    image_slow.save("./image_slow.png")?;
    image_fast.save("./image_fast.png")?;

    Ok(())
}
