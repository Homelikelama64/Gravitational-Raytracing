use cgmath::{vec3, InnerSpace, MetricSpace, Vector3, Zero};
use rayon::prelude::*;
use simple_video::*;
use std::{
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
struct Body {
    pos: Vector3<f32>,
    vel: Vector3<f32>,
    radius: f32,
    color: ColorF32,
    mass: f32,
}

fn trace_ray(
    x: f32,
    y: f32,
    aspect: f32,
    bodies_path: &Vec<Vec<Body>>,
    max_distance: f32,
    light_speed: f32,
    gravity_strength: f32,
    dt: f32,
) -> ColorF32 {
    // let gravity_strength = 1.0;
    // let light_speed = 1.0;

    let mut photon_pos = vec3(0.0, 0.0, -10.0);
    let mut photon_dir =
        vec3((x * 2.0 - 1.0) * aspect, y * 2.0 - 1.0, 1.0).normalize_to(light_speed);

    // let max_distance = 40.0;
    // let dt = 0.05;

    let iter_count = (max_distance / dt / light_speed).ceil() as usize;
    let simulation_length = iter_count as f32 * dt;

    let mut elapsed_time = 0.0;
    let mut total_force_of_gravity = Vector3::zero();
    for _ in 0..iter_count {
        elapsed_time += dt;
        let time = simulation_length - elapsed_time;
        let simulation_percent = time / simulation_length;
        let bodies =
            &bodies_path[f32::floor(bodies_path.len() as f32 * simulation_percent) as usize];
        for body in bodies {
            if photon_pos.distance2(body.pos) < body.radius * body.radius {
                return body.color;
            }

            if body.mass != 0.0 {
                let gravity = (gravity_strength * body.mass) / body.pos.distance2(photon_pos);

                // black hole pull is greater than light speed, the light cannot escape
                if gravity > light_speed {
                    return ColorF32 {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    };
                }
                total_force_of_gravity += (body.pos - photon_pos).normalize() * gravity * dt;

                photon_dir += (body.pos - photon_pos).normalize() * gravity * dt;
            }
        }

        photon_dir = photon_dir.normalize_to(light_speed);

        photon_pos += photon_dir * dt;
    }

    return ColorF32 {
        r: 0.1,
        g: 0.1,
        b: 0.1,
    };
}

#[allow(dead_code)]
trait Lerp {
    fn lerp(a: Self, b: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        a + (b - a) * t
    }
}

impl Lerp for Vector3<f32> {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        vec3(
            f32::lerp(a.x, b.x, t),
            f32::lerp(a.y, b.y, t),
            f32::lerp(a.z, b.z, t),
        )
    }
}

fn physics(
    bodies: &Vec<Body>,
    dt: f32,
    max_distance: f32,
    light_speed: f32,
    gravity_strength: f32,
) -> Vec<Vec<Body>> {
    let mut bodies_path: Vec<Vec<Body>> = vec![bodies.clone()];
    let iter_count = (max_distance / dt / light_speed).ceil() as usize;
    for s in 0..iter_count {
        let mut new_bodies: Vec<Body> = Vec::with_capacity(bodies.len());
        for i in 0..bodies.len() {
            let mut a_body =
                bodies_path[i32::clamp(s as i32 - 1, 0, iter_count as i32) as usize][i].clone();
            for j in 0..bodies.len() {
                if i != j {
                    let b_body = bodies_path
                        [i32::clamp(s as i32 - 1, 0, iter_count as i32) as usize][j]
                        .clone();
                    if b_body.mass != 0.0 {
                        a_body.vel += (b_body.pos - a_body.pos).normalize()
                            * (gravity_strength * b_body.mass
                                / (b_body.pos - a_body.pos).magnitude2())
                            * dt
                    }
                }
            }
            a_body.pos += a_body.vel * dt;
            new_bodies.push(a_body);
        }
        bodies_path.push(new_bodies);
    }
    return bodies_path;
}

pub fn trace_rays(pixels: &mut [ColorF32], width: usize, height: usize) {

    let pixel_count = pixels.len();
    assert_eq!(pixel_count, width * height);
    let aspect = width as f32 / height as f32;

    let dt = 0.01;
    let max_distance = 40.0;
    let light_speed = 1.0;
    let gravity_strength = 1.0;

    let bodies: Vec<Body> = vec![
        Body {
            pos: vec3(0.0, 0.0, 0.0),
            vel: vec3(0.0, 0.0, 0.0),
            radius: 0.0,
            color: ColorF32 {
                r: 1.0,
                g: 1.0,
                b: 0.0,
            },
            mass: 1.0,
        },
        Body {
            pos: vec3(0.0, 1.2, 0.0),
            vel: vec3(1.0, 0.0, 0.0),
            radius: 0.05,
            color: ColorF32 {
                r: 1.0,
                g: 1.0,
                b: 0.0,
            },
            mass: 0.0,
        },
    ];
    println!("Simulating Bodies");
    let bodies_path = physics(&bodies, dt, max_distance, light_speed, gravity_strength);
    println!("Done.");
    println!("Simulating Light");
    //dbg!(&bodies_path);

    let start = Instant::now();
    let completed_pixels = AtomicUsize::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            pixels.par_iter_mut().enumerate().for_each(|(i, color)| {
                let (x, y) = (i % width, i / width);

                let samples_resolution = 1;

                let mut samples_color = ColorF32 {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                };
                for y_offset in 0..samples_resolution {
                    for x_offset in 0..samples_resolution {
                        let (x, y) = (
                            (x as f32 + (x_offset as f32 + 0.5) / samples_resolution as f32)
                                / width as f32,
                            (y as f32 + (y_offset as f32 + 0.5) / samples_resolution as f32)
                                / height as f32,
                        );
                        samples_color += trace_ray(
                            x,
                            y,
                            aspect,
                            &bodies_path,
                            max_distance,
                            light_speed,
                            gravity_strength,
                            dt,
                        );
                    }
                }
                *color = ColorF32 {
                    r: samples_color.r / (samples_resolution * samples_resolution) as f32,
                    g: samples_color.g / (samples_resolution * samples_resolution) as f32,
                    b: samples_color.b / (samples_resolution * samples_resolution) as f32,
                };

                completed_pixels.fetch_add(1, Ordering::Relaxed);
            });
        });

        loop {
            let progress = completed_pixels.load(Ordering::Relaxed);
            let total_time = start.elapsed();
            print!(
                "\rProgress: {:.1}%, Estimated time remaining: {:.1}s            ",
                (progress as f32 / pixel_count as f32) * 100.0,
                total_time.as_secs_f32() / (progress as f32 / pixel_count as f32)
                    - total_time.as_secs_f32()
            );
            std::io::stdout().flush().unwrap();
            if progress >= pixel_count {
                println!();
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    assert_eq!(completed_pixels.into_inner(), pixel_count);
}
