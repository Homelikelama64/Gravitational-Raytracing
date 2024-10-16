use derive_more::derive::{Add, AddAssign};

#[derive(Debug, Clone, Copy, Add, AddAssign)]
pub struct ColorF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Clone, Copy, Add, AddAssign)]
pub struct ColorU8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Frame {
    pixels: Vec<ColorU8>,
}

pub struct Video {
    width: u16,
    height: u16,
    frame_count: u32,
    fps: u8,
    frames: Vec<Frame>,
}

impl Video {
    fn add_frame(&mut self, pixels: &Vec<ColorF32>) {
        let mut new_frame = Frame { pixels: vec![] };
        for pixel in pixels {
            new_frame.pixels.push(pixel.to_u8());
        }
        self.frames.push(new_frame);
    }
}

impl ColorF32 {
    fn to_u8(&self) -> ColorU8 {
        return ColorU8 {
            r: (self.r * 255.0) as u8,
            g: (self.g * 255.0) as u8,
            b: (self.b * 255.0) as u8,
        };
    }
}
