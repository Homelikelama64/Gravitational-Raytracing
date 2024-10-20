use raylib::prelude::*;
use simple_video::*;
use std::{
    env,
    ffi::{CStr, CString},
    fmt::format,
    time::SystemTime,
};

const BOARDER: i32 = 100;

fn main() {
    let args: Vec<String> = dbg!(env::args().collect());

    let (mut rl, thread) = raylib::init()
        .size(1080, 720)
        .title("Simple Video")
        .resizable()
        .build();
    let video = read_video_from_file(&args[1]).unwrap();

    let mut percentage: f32 = 0.0;

    while !rl.window_should_close() {
        let width: i32 = rl.get_screen_width();
        let height: i32 = rl.get_screen_height();
        let mut scale = (height as f32 - BOARDER as f32) / video.height() as f32;
        if video.width() as f32 * scale > width as f32 - 40.0 {
            scale = (width as f32 - 40.0) / video.width() as f32
        }

        let dt = rl.get_frame_time();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::new(51, 51, 51, 255));

        let vid_time = video.frame_count() as f32 * video.fps() as f32;
        d.gui_slider(
            Rectangle {
                x: 20.0,
                y: height as f32 - BOARDER as f32 + 20.0,
                width: width as f32 - 50.0,
                height: 20.0,
            },
            Some(c"0:00"),
            Some(c"0:00"),
            &mut percentage,
            0.0,
            1.0,
        );

        let frame = video
            .get_frame(
                (percentage * video.frame_count() as f32)
                    .clamp(0 as f32, video.frame_count() as f32 - 1.0)
                    .floor() as usize,
            )
            .unwrap();
        let time = SystemTime::now();
        let (sender, receiver) = std::sync::mpsc::channel::<(Image,i32)>();
        rayon::scope(|s| {
            let screen_width = (video.width() as f32 * scale) as i32;
            let screen_height = (video.height() as f32 * scale) as i32;
            for x in 0..screen_width {
                let video = &video;
                s.spawn(move |_| {
                    let mut texture =
                        Image::gen_image_color(1, screen_height, Color::new(255, 0, 255, 255));
                    for y in 0..screen_height {
                        let index = x + y * screen_width;
                        let sample_index = ((index % screen_width) as f32 / scale).floor() as i32
                            + (((index / screen_width) as f32 / scale).floor() as i32
                                * video.width() as i32);
                        let coloru8 = frame[sample_index as usize];
                        // let x = x + ((width as f32 - video.width() as f32 * scale) / 2.0) as i32;
                        // let y = y
                        //     + ((height as f32 - BOARDER as f32 + 20.0
                        //         - video.height() as f32 * scale)
                        //         / 2.0) as i32;
                    }
                    sender.send((texture,x)).unwrap()
                });
            }
            drop(sender);
            for (image,x) in receiver {

            }
        });
        dbg!(time.elapsed().unwrap().as_millis());

        d.draw_text(
            format!("FPS: {:.2}", 1.0 / dt).as_str(),
            10,
            20,
            18,
            Color::new(255, 255, 255, 255),
        );
    }
}
