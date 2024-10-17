use ray_tracing::trace_rays;
use simple_video::*;
use std::{
    fs::File, io::{BufWriter, Write}
};

fn main() {
    let (width, height) = (720, 480);
    let mut pixels = vec![
        ColorF32 {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        width * height
    ];

    trace_rays(&mut pixels, width, height);
    println!("Done.");

    println!("Writing Image...");
    let mut file = BufWriter::new(File::create("output.ppm").unwrap());
    writeln!(file, "P6").unwrap();
    writeln!(file, "{width} {height}").unwrap();
    writeln!(file, "255").unwrap();
    for y in (0..height).rev() {
        for x in 0..width {
            let index = x + y * width;
            let ColorF32 { r, g, b } = pixels[index];
            let (r, g, b) = (
                (r.clamp(0.0, 1.0) * 255.0) as u8,
                (g.clamp(0.0, 1.0) * 255.0) as u8,
                (b.clamp(0.0, 1.0) * 255.0) as u8,
            );
            file.write_all(&[r, g, b]).unwrap();
        }
    }
    file.flush().unwrap();

    let mut video: Video = Video {
        width: width as u16,
        height: height as u16,
        frame_count: 1,
        fps: 1,
        frames: vec![],
        version: 0
    };
    video.add_frame(&pixels);
    video.write_to_file("output".to_string());

    read_file("C:/dev/Rust/Gravitational-Raytracing/output.simplevideo".to_string());

    println!("Done.");
}
