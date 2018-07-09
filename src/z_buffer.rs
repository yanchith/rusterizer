use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct ZBuffer {
    width: u32,
    height: u32,
    buffer: Vec<f64>,
}

impl ZBuffer {
    pub fn new(width: u32, height: u32, depth: f64) -> ZBuffer {
        ZBuffer {
            width,
            height,
            buffer: vec![depth; (width * height) as usize],
        }
    }

    pub fn get(&self, x: u32, y: u32) -> f64 {
        self.buffer[(y * self.width + x) as usize]
    }

    pub fn set(&mut self, x: u32, y: u32, depth: f64) {
        self.buffer[(y * self.width + x) as usize] = depth;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn into_vec(self) -> Vec<f64> {
        self.buffer
    }
}

impl fmt::Display for ZBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text =
            String::with_capacity((18 * self.width * self.height) as usize);
        for chunk in self.buffer.chunks(self.width as usize) {
            for item in chunk {
                text.push_str(&format!(" {:03} ", item));
            }
            text.push('\n');
        }
        write!(f, "{}", text)?;
        Ok(())
    }
}
