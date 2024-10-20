use cgmath::vec3;
use ray_tracing::{trace_rays, Body, Universe};
use simple_video::*;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

fn main() {
    let (width, height) = (500, 250);
    let mut pixels = vec![
        ColorF32 {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        width * height
    ];
    let mut universe = Universe::new(
        0.0,
        40.0,
        vec![Body {
            pos: vec3(-50.0, 0.0, 0.0),
            vel: vec3(5.0, 0.0, 0.0),
            radius: 1.0,
            color: ColorF32 {
                r: 1.0,
                g: 1.0,
                b: 0.0,
            },
            mass: 0.0,
        }],
        20.0,
        1.0,
        1.0,
        0.01,
    );
    let mut vid = Video::new(width as u32, height as u32, 20);

    for i in 0..(universe.animation_length * vid.fps() as f32) as usize {
        let time = i as f32 * (1.0 / vid.fps() as f32);
        universe.time = time;
        trace_rays(
            &mut pixels,
            width,
            height,
            &universe
        );
        vid.append_frame(pixels.iter().copied().map(Into::into));
        println!("{:.2}%",i as f32 / (universe.animation_length * vid.fps() as f32) * 100.0);
    }
    write_video_to_file(&vid, "TestFile.simvid").unwrap();

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
