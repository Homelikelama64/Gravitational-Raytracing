use ray_tracing::{trace_rays, Color};
use std::{
    fs::File,
    io::{BufWriter, Write},
};

fn main() {
    let (width, height) = (1080, 720);
    let mut pixels = vec![
        Color {
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
            let Color { r, g, b } = pixels[index];
            let (r, g, b) = (
                (r.clamp(0.0, 1.0) * 255.0) as u8,
                (g.clamp(0.0, 1.0) * 255.0) as u8,
                (b.clamp(0.0, 1.0) * 255.0) as u8,
            );
            file.write_all(&[r, g, b]).unwrap();
        }
    }
    file.flush().unwrap();

    println!("Done.");
}
