use chrono::Local;
use ray_tracing::{trace_rays, StartConditions, Universe};
use simple_video::*;
use std::{fs::File, io::Read, path::Path, time::SystemTime};

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut config = File::open(&path).unwrap();
    let mut config_string: String = "".to_string();
    let _ = config.read_to_string(&mut config_string);

    let start_conditions: StartConditions = serde_json::from_str(&config_string).unwrap();

    let (width, height) = (start_conditions.width, start_conditions.height);
    let mut pixels = vec![
        ColorF32 {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        };
        width * height
    ];
    let mut universe = Universe::new(&start_conditions);

    println!("Rendering Video at: {}", { Local::now().to_rfc2822() });
    let mut vid = Video::new(width as u32, height as u32, start_conditions.fps as u8);
    let start = SystemTime::now();
    for i in 0..(universe.animation_length * vid.fps() as f32) as usize {
        let time = i as f32 * (1.0 / vid.fps() as f32);
        universe.time = time;
        trace_rays(&mut pixels, width, height, &universe, vid.fps(), i, start);
        vid.append_frame(pixels.iter().copied().map(Into::into));
    }
    println!("\nDone at: {}", { Local::now().to_rfc2822() });
    println!("Writing Video");
    //let name = Path::new(&path).file_name().unwrap().to_str().unwrap();
    let name = Path::new(&path).file_name().unwrap().to_str().unwrap();
    let mut trimmed_path = path.clone();
    if name.len() < path.len() {
        let _ = trimmed_path.split_off(path.len() - name.len() - 1);
        trimmed_path += "/"
    }else {
        trimmed_path = "".to_string();
    }
    write_video_to_file(&vid, trimmed_path + "output.simvid").unwrap();
    println!("Done.");
}
