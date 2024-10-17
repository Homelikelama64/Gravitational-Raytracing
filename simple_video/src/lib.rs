use derive_more::derive::{Add, AddAssign};
use std::io::Bytes;

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

#[allow(unused)]
pub struct Frame {
    pub pixels: Vec<ColorU8>,
}

#[allow(unused)]
pub struct Video {
    pub width: u16,
    pub height: u16,
    pub frame_count: u32,
    pub fps: u8,
    pub frames: Vec<Frame>,
    pub version: u8,
}

#[allow(unused)]
impl Video {
    pub fn add_frame(&mut self, pixels: &Vec<ColorF32>) {
        let mut new_frame = Frame { pixels: vec![] };
        for pixel in pixels {
            new_frame.pixels.push(pixel.to_u8());
        }
        self.frames.push(new_frame);
    }
    pub fn write_to_file(&self, path: String) {
        let mut bytes: Vec<u8> = vec![];
        let width = u16_to_u8s(self.width);
        let height = u16_to_u8s(self.height);
        bytes.push('s' as u8);
        bytes.push('i' as u8);
        bytes.push('m' as u8);
        bytes.push('v' as u8);
        bytes.push('i' as u8);
        bytes.push('d' as u8);
        bytes.push('v' as u8);
        bytes.push(self.version);
        bytes.push(width.0);
        bytes.push(width.1);
        bytes.push(height.0);
        bytes.push(height.1);
        let frame_count = u32_to_u8s(self.frame_count);
        bytes.push(frame_count.0);
        bytes.push(frame_count.1);
        bytes.push(frame_count.2);
        bytes.push(frame_count.3);
        bytes.push(self.fps);
        for frame in &self.frames {
            for pixel in &frame.pixels {
                bytes.append(&mut pixel.to_bytes());
            }
        }
        std::fs::write(path + ".simplevideo", bytes).unwrap()
    }
}

pub fn read_file(path: String) {
    let bytes = std::fs::read(path).unwrap();
    let data = bytes.split_at(6);
    let data = (data.0.to_vec(),data.1.to_vec());
    if String::from_utf8(data.0.clone()).unwrap() == "simvid" {
        let version = &data.1[1];
        let width = u8s_to_u16((&data.1[2],&data.1[3]));
        let hieght = u8s_to_u16((&data.1[4],&data.1[5]));
    }
}

impl ColorU8 {
    pub fn to_bytes(&self) -> Vec<u8> {
        return vec![self.r, self.g, self.b];
    }
}

#[allow(unused)]
impl ColorF32 {
    pub fn to_u8(&self) -> ColorU8 {
        return ColorU8 {
            r: (self.r * 255.0) as u8,
            g: (self.g * 255.0) as u8,
            b: (self.b * 255.0) as u8,
        };
    }
}

#[allow(unused)]
fn u16_to_u8s(value: u16) -> (u8, u8) {
    return ((value >> 8) as u8, value as u8);
}

#[allow(unused)]
fn u8s_to_u16(tokens: (&u8,&u8)) -> u16 {
    let value:u16 = 0;
    return value + *tokens.1 as u16 + ((*tokens.0 as u16) << 8);
}

#[allow(unused)]
fn u32_to_u8s(value: u32) -> (u8, u8, u8, u8) {
    return (
        (value >> 24) as u8,
        (value >> 16) as u8,
        (value >> 8) as u8,
        (value) as u8,
    );
}
