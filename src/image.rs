use std::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Color(u8, u8, u8, u8);

impl Color {
    pub fn red() -> Color {
        Color(255, 0, 0, 255)
    }

    pub fn green() -> Color {
        Color(0, 255, 0, 255)
    }

    pub fn blue() -> Color {
        Color(0, 0, 255, 255)
    }

    pub fn black() -> Color {
        Color(0, 0, 0, 255)
    }

    pub fn white() -> Color {
        Color(255, 255, 255, 255)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{:03},{:03},{:03},{:03}]",
            self.0, self.1, self.2, self.3
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Image {
    width: usize,
    height: usize,
    buffer: Vec<Color>,
}

impl Image {
    pub fn new(width: usize, height: usize, color: Color) -> Image {
        Image {
            width,
            height,
            buffer: vec![color; width * height],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        self.buffer[y * self.width + x] = color;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text = String::with_capacity(18 * self.width * self.height);
        for chunk in self.buffer.chunks(self.width) {
            for item in chunk {
                text.push_str(&format!("{}", item));
            }
            text.push('\n');
        }
        write!(f, "{}", text)?;
        Ok(())
    }
}
