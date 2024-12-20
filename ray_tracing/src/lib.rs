use cgmath::{vec3, InnerSpace, MetricSpace, Vector3};
use chrono::{Local, TimeDelta};
use rayon::prelude::*;
use simple_video::*;
use std::{
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant, SystemTime},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub pos: Vector3<f32>,
    pub vel: Vector3<f32>,
    pub radius: f32,
    pub color: ColorF32,
    pub mass: f32,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Universe {
    pub time: f32,
    pub animation_length: f32,
    pub bodies_path: Vec<Vec<Body>>,
    pub max_distance: f32,
    pub light_speed: f32,
    pub gravity_strength: f32,
    pub dt: f32,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StartConditions {
    pub width: usize,
    pub height: usize,
    pub fps: usize,
    pub time: f32,
    pub animation_length: f32,
    pub bodies: Vec<Body>,
    pub max_distance: f32,
    pub light_speed: f32,
    pub gravity_strength: f32,
    pub dt: f32,
}

impl Universe {
    pub fn new(start_conditions: &StartConditions) -> Universe {
        let mut bodies_path: Vec<Vec<Body>> = vec![start_conditions.bodies.clone()];

        let mut current_bodies = start_conditions.bodies.clone();
        for body in &mut current_bodies {
            body.vel = -body.vel
        }
        for _ in
            0..(start_conditions.max_distance / start_conditions.dt / start_conditions.light_speed)
                .ceil() as usize
        {
            for a in 0..current_bodies.len() {
                for b in 0..current_bodies.len() {
                    if a != b {
                        if current_bodies[b].mass != 0.0 {
                            let vel = (current_bodies[b].pos - current_bodies[a].pos).normalize()
                                * (start_conditions.gravity_strength * current_bodies[b].mass
                                    / (current_bodies[b].pos - current_bodies[a].pos).magnitude2());
                            current_bodies[a].vel += vel * start_conditions.dt;
                        }
                    }
                }
                let vel = current_bodies[a].vel * start_conditions.dt;
                current_bodies[a].pos += vel;
            }
            let mut new_vec: Vec<Vec<Body>> = vec![current_bodies.clone()];
            new_vec.append(&mut bodies_path.clone());
            bodies_path = new_vec.clone();
        }

        let mut current_bodies = start_conditions.bodies.clone();

        for _ in 0..(start_conditions.animation_length / start_conditions.dt) as usize {
            for a in 0..current_bodies.len() {
                for b in 0..current_bodies.len() {
                    if a != b {
                        if current_bodies[b].mass != 0.0 {
                            let vel = (current_bodies[b].pos - current_bodies[a].pos).normalize()
                                * (start_conditions.gravity_strength * current_bodies[b].mass
                                    / (current_bodies[b].pos - current_bodies[a].pos).magnitude2());
                            current_bodies[a].vel += vel * start_conditions.dt;
                        }
                    }
                }
                let vel = current_bodies[a].vel * start_conditions.dt;
                current_bodies[a].pos += vel;
            }
            bodies_path.append(&mut vec![current_bodies.clone()]);
        }

        return Universe {
            time: start_conditions.time,
            animation_length: start_conditions.animation_length,
            bodies_path: bodies_path,
            max_distance: start_conditions.max_distance,
            light_speed: start_conditions.light_speed,
            gravity_strength: start_conditions.gravity_strength,
            dt: start_conditions.dt,
        };
    }

    pub fn light_iter_count(&self) -> usize {
        (self.max_distance / self.dt / self.light_speed).ceil() as usize
    }
    pub fn light_simulation_length(&self) -> f32 {
        self.light_iter_count() as f32 * self.dt
    }
    pub fn time_percent(&self, time: f32) -> f32 {
        (time + self.light_simulation_length())
            / (self.animation_length + self.light_simulation_length())
    }
    pub fn get_bodies_at_time_percent(&self, time: f32) -> &Vec<Body> {
        &self.bodies_path
            [((self.bodies_path.len() as f32 * time) as usize).clamp(0, self.bodies_path.len() - 1)]
    }
}

fn trace_ray(x: f32, y: f32, aspect: f32, universe: &Universe) -> ColorF32 {
    let mut photon_pos = vec3(0.0, 0.0, 0.0);
    let mut photon_dir =
        vec3((x * 2.0 - 1.0) * aspect, y * 2.0 - 1.0, 1.0).normalize_to(universe.light_speed);

    let mut elapsed = 0.0;
    for i in 0..universe.light_iter_count() {
        elapsed -= universe.dt;
        let time = universe.time + elapsed;

        let iterations_left = universe.light_iter_count() - i;
        let max_distance = iterations_left as f32 * universe.light_speed * universe.dt;

        let mut close_to_body = false;
        for body in universe.get_bodies_at_time_percent(universe.time_percent(time)) {
            let dist = photon_pos.distance(body.pos);
            if dist < max_distance {
                close_to_body = true;
            }
            if dist * dist < body.radius * body.radius {
                return body.color;
            }

            if body.mass != 0.0 {
                let a = (4.0 * universe.gravity_strength * body.mass)
                    / ((universe.light_speed * universe.light_speed)
                        * dist);
                let tug = a * (body.pos - photon_pos).normalize();
                photon_dir += tug * universe.dt;
                photon_dir = photon_dir.normalize_to(universe.light_speed);
            }
        }
        if close_to_body == false {
            break;
        }
        photon_dir = photon_dir.normalize_to(universe.light_speed);

        photon_pos += photon_dir * universe.light_speed * universe.dt;
    }

    return ColorF32 {
        r: 0.1,
        g: 0.1,
        b: 0.1,
    };
}

pub fn trace_rays(
    pixels: &mut [ColorF32],
    width: usize,
    height: usize,
    universe: &Universe,
    fps: u8,
    i: usize,
    start: SystemTime,
) {
    let pixel_count = pixels.len();
    assert_eq!(pixel_count, width * height);
    let aspect = width as f32 / height as f32;

    let start_frame = Instant::now();
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
                        samples_color += trace_ray(x, y, aspect, &universe);
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
            let total_time = start_frame.elapsed();

            let time = start.elapsed().unwrap();
            let eta = time.as_secs_f32() as f32
                / ((i as f32 + (progress as f32 / pixel_count as f32))
                    / (universe.animation_length * fps as f32))
                - time.as_secs_f32();
            let time_spent_rendering = time.as_secs_f32();
            let finish_time = if i == 0 {
                Local::now()
            } else {
                Local::now()
                    .checked_add_signed(TimeDelta::seconds(eta as i64))
                    .unwrap()
            };
            print!(
                "\rProgress: {:.1}%, Time spent on frame: {:.1}s, Animation Progress: {:.0}/{:.0} {:.2}%,Total Time Left: {:.0}:{:.0}:{:.0}, Time Rendering: {:.0}:{:.0}:{:.0}, Finishes At: {}            ",
                (progress as f32 / pixel_count as f32) * 100.0,
                total_time.as_secs_f32(),
                i,
                universe.animation_length * fps as f32,
                (i as f32 + (progress as f32 / pixel_count as f32)) / (universe.animation_length * fps as f32) * 100.0,
                (eta / 60.0 /60.0).floor(),
                (eta / 60.0 % 60.0).floor(),
                (eta % 60.0).floor(),
                (time_spent_rendering / 60.0 / 60.0).floor(),
                (time_spent_rendering / 60.0 % 60.0).floor(),
                (time_spent_rendering % 60.0).floor(),
                finish_time.to_rfc2822(),
            );
            std::io::stdout().flush().unwrap();
            if progress >= pixel_count {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    assert_eq!(completed_pixels.into_inner(), pixel_count);
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

// fn physics(
//     bodies: &Vec<Body>,
//     dt: f32,
//     max_distance: f32,
//     light_speed: f32,
//     gravity_strength: f32,
// ) -> Vec<Vec<Body>> {
//     let mut bodies_path: Vec<Vec<Body>> = vec![bodies.clone()];
//     let iter_count = (max_distance / dt / light_speed).ceil() as usize;
//     for s in 0..iter_count {
//         let mut new_bodies: Vec<Body> = Vec::with_capacity(bodies.len());
//         for i in 0..bodies.len() {
//             let mut a_body =
//                 bodies_path[i32::clamp(s as i32 - 1, 0, iter_count as i32) as usize][i].clone();
//             for j in 0..bodies.len() {
//                 if i != j {
//                     let b_body = bodies_path
//                         [i32::clamp(s as i32 - 1, 0, iter_count as i32) as usize][j]
//                         .clone();
//                     if b_body.mass != 0.0 {
//                         a_body.vel += (b_body.pos - a_body.pos).normalize()
//                             * (gravity_strength * b_body.mass
//                                 / (b_body.pos - a_body.pos).magnitude2())
//                             * dt
//                     }
//                 }
//             }
//             a_body.pos += a_body.vel * dt;
//             new_bodies.push(a_body);
//         }
//         bodies_path.push(new_bodies);
//     }
//     return bodies_path;
// }

// fn trace_ray(
//     x: f32,
//     y: f32,
//     aspect: f32,
//     bodies_path: &Vec<Vec<Body>>,
//     max_distance: f32,
//     light_speed: f32,
//     gravity_strength: f32,
//     dt: f32,
// ) -> ColorF32 {
//     // let gravity_strength = 1.0;
//     // let light_speed = 1.0;

//     let mut photon_pos = vec3(0.0, 0.0, -10.0);
//     let mut photon_dir =
//         vec3((x * 2.0 - 1.0) * aspect, y * 2.0 - 1.0, 1.0).normalize_to(light_speed);

//     // let max_distance = 40.0;
//     // let dt = 0.05;

//     let iter_count = (max_distance / dt / light_speed).ceil() as usize;
//     let simulation_length = iter_count as f32 * dt;

//     let mut elapsed_time = 0.0;
//     let mut total_force_of_gravity = Vector3::zero();
//     for _ in 0..iter_count {
//         elapsed_time += dt;
//         let time = simulation_length - elapsed_time;
//         let simulation_percent = time / simulation_length;
//         let bodies =
//             &bodies_path[f32::floor(bodies_path.len() as f32 * simulation_percent) as usize];
//         for body in bodies {
//             if photon_pos.distance2(body.pos) < body.radius * body.radius {
//                 return body.color;
//             }

//             if body.mass != 0.0 {
//                 let gravity = (gravity_strength * body.mass) / body.pos.distance2(photon_pos);

//                 // black hole pull is greater than light speed, the light cannot escape
//                 if gravity > light_speed {
//                     return ColorF32 {
//                         r: 0.0,
//                         g: 0.0,
//                         b: 0.0,
//                     };
//                 }
//                 total_force_of_gravity += (body.pos - photon_pos).normalize() * gravity * dt;

//                 photon_dir += (body.pos - photon_pos).normalize() * gravity * dt;
//             }
//         }

//         photon_dir = photon_dir.normalize_to(light_speed);

//         photon_pos += photon_dir * dt;
//     }

//     return ColorF32 {
//         r: 0.1,
//         g: 0.1,
//         b: 0.1,
//     };
// }
