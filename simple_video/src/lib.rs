use bytemuck::{Pod, Zeroable};
use derive_more::derive::{Add, AddAssign};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    ops::{Index, IndexMut},
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroable, Pod)]
#[repr(C)]
pub struct ColorU8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, Add, AddAssign, serde::Serialize,serde::Deserialize)]
pub struct ColorF32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<ColorF32> for ColorU8 {
    fn from(ColorF32 { r, g, b }: ColorF32) -> Self {
        ColorU8 {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Video {
    width: u32,
    height: u32,
    fps: u8,
    pixels: Vec<ColorU8>,
}

impl Video {
    pub fn new(width: u32, height: u32, fps: u8) -> Video {
        assert_ne!(width, 0);
        assert_ne!(height, 0);
        assert_ne!(fps, 0);
        Video {
            width,
            height,
            fps,
            pixels: vec![],
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn fps(&self) -> u8 {
        self.fps
    }

    pub fn frame_count(&self) -> u32 {
        u32::try_from(self.pixels.len() / self.width as usize / self.height as usize)
            .expect("frame count should fit in `u32`")
    }

    pub fn append_frame(
        &mut self,
        frame: impl IntoIterator<Item = ColorU8, IntoIter: ExactSizeIterator>,
    ) {
        let frame = frame.into_iter();
        assert_eq!(
            frame.len(),
            self.width as usize * self.height as usize,
            "`frame` should have the correct number of colors for {} * {}",
            self.width,
            self.height,
        );
        self.pixels.extend(frame);
    }

    pub fn remove_frame(&mut self, index: usize) -> std::vec::Drain<'_, ColorU8> {
        let length = self.width as usize * self.height as usize;
        self.pixels.drain(index * length..(index + 1) * length)
    }

    pub fn get_frame(&self, index: usize) -> Option<&[ColorU8]> {
        let length = self.width as usize * self.height as usize;
        self.pixels.get(index * length..)?.get(..length)
    }

    pub fn get_frame_mut(&mut self, index: usize) -> Option<&mut [ColorU8]> {
        let length = self.width as usize * self.height as usize;
        self.pixels.get_mut(index * length..)?.get_mut(..length)
    }
}

impl Index<usize> for Video {
    type Output = [ColorU8];

    fn index(&self, index: usize) -> &Self::Output {
        self.get_frame(index).expect("`index` should be in-bounds")
    }
}

impl IndexMut<usize> for Video {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_frame_mut(index)
            .expect("`index` should be in-bounds")
    }
}

const MAGIC_BYTES: [u8; 6] = *b"simvid";

pub fn write_video(video: &Video, mut f: impl Write) -> std::io::Result<()> {
    f.write_all(&MAGIC_BYTES)?;
    f.write_all(&u32::to_be_bytes(video.width))?;
    f.write_all(&u32::to_be_bytes(video.height))?;
    f.write_all(&u32::to_be_bytes(video.frame_count()))?;
    f.write_all(std::slice::from_ref(&video.fps))?;
    f.write_all(bytemuck::cast_slice(&video.pixels))?;
    Ok(())
}

pub fn write_video_to_file(video: &Video, path: impl AsRef<Path>) -> std::io::Result<()> {
    write_video(video, BufWriter::new(File::create(path)?))
}

pub fn read_video(mut f: impl Read) -> std::io::Result<Video> {
    fn read_u32(mut f: impl Read) -> std::io::Result<u32> {
        let mut value = [0; size_of::<u32>()];
        f.read_exact(&mut value)?;
        Ok(u32::from_be_bytes(value))
    }

    let mut magic = [0; 6];
    f.read_exact(&mut magic)?;
    assert_eq!(magic, MAGIC_BYTES);

    let width = read_u32(&mut f)?;
    assert_ne!(width, 0);

    let height = read_u32(&mut f)?;
    assert_ne!(width, 0);

    let frame_count = read_u32(&mut f)?;

    let mut fps = 0;
    f.read_exact(std::slice::from_mut(&mut fps))?;
    assert_ne!(fps, 0);

    let mut pixels =
        vec![ColorU8 { r: 0, g: 0, b: 0 }; width as usize * height as usize * frame_count as usize];
    f.read_exact(bytemuck::cast_slice_mut(&mut pixels))?;

    Ok(Video {
        width,
        height,
        fps,
        pixels,
    })
}

pub fn read_video_from_file(path: impl AsRef<Path>) -> std::io::Result<Video> {
    read_video(BufReader::new(File::open(path)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_size_image() {
        let video = Video::new(1, 2, 1);

        let mut bytes = vec![];
        write_video(&video, &mut bytes).unwrap();

        let read_video = read_video(bytes.as_slice()).unwrap();
        assert_eq!(video, read_video);
    }

    #[test]
    fn one_size_image() {
        let mut video = Video::new(1, 2, 1);
        video.append_frame([ColorU8 { r: 1, g: 2, b: 3 }, ColorU8 { r: 4, g: 5, b: 6 }]);

        let mut bytes = vec![];
        write_video(&video, &mut bytes).unwrap();

        let read_video = read_video(bytes.as_slice()).unwrap();
        assert_eq!(video, read_video);
    }
}
