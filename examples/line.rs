extern crate failure;
extern crate image;
extern crate rusterizer;

use failure::Error;
use image::{ImageBuffer, Rgba};

use rusterizer::{line_3, line_4, line_5};

fn black() -> Rgba<u8> {
    Rgba([0, 0, 0, 255])
}

fn red() -> Rgba<u8> {
    Rgba([255, 0, 0, 255])
}

fn main() -> Result<(), Error> {
    let mut image3 = ImageBuffer::from_pixel(6, 6, black());
    let mut image4 = ImageBuffer::from_pixel(6, 6, black());
    let mut image5 = ImageBuffer::from_pixel(6, 6, black());

    line_3(2, 1, 5, 5, &mut image3, red());
    line_3(5, 5, 1, 4, &mut image3, red());
    line_3(1, 4, 2, 1, &mut image3, red());

    line_4(2, 1, 5, 5, &mut image4, red());
    line_4(5, 5, 1, 4, &mut image4, red());
    line_4(1, 4, 2, 1, &mut image4, red());

    line_5(2, 1, 5, 5, &mut image5, red());
    line_5(5, 5, 1, 4, &mut image5, red());
    line_5(1, 4, 2, 1, &mut image5, red());

    image3.save("./image3.png")?;
    image4.save("./image4.png")?;
    image5.save("./image5.png")?;

    Ok(())
}
