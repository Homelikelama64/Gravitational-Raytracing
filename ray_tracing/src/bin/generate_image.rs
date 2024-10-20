use cgmath::vec3;
use ray_tracing::{trace_rays, Body, Universe};
use simple_video::*;
use std::{
    fs::File,
    io::{BufWriter, Write},
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

    trace_rays(
        &mut pixels,
        width,
        height,
        Universe::new(
            1.0,
            1.0,
            vec![Body {
                pos: vec3(0.0, 0.0, 0.0),
                vel: vec3(0.0, 0.1, 0.0),
                radius: 1.0,
                color: ColorF32 {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                },
                mass: 1.0,
            }],
            40.0,
            1.0,
            1.0,
            0.01,
        ),
    );
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

    //let mut vid = Video::new(width as u32, height as u32, 1);
    //vid.append_frame(pixels.iter().copied().map(Into::into));
    //vid.append_frame(pixels.iter().copied().map(Into::into));
    //vid.append_frame(pixels.iter().copied().map(Into::into));
    //vid.append_frame(pixels.iter().copied().map(Into::into));
    //write_video_to_file(&vid, "output.simvid").unwrap();

    println!("Done.");
}
