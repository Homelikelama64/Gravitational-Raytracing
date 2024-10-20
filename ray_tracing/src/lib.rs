use cgmath::{vec3, InnerSpace, MetricSpace, Vector3, Zero};
use rayon::prelude::*;
use simple_video::*;
use std::{
    io::Write,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct Body {
    pub pos: Vector3<f32>,
    pub vel: Vector3<f32>,
    pub radius: f32,
    pub color: ColorF32,
    pub mass: f32,
}

pub struct Universe {
    pub time: f32,
    pub animation_length: f32,
    pub bodies_path: Vec<Vec<Body>>,
    pub max_distance: f32,
    pub light_speed: f32,
    pub gravity_strength: f32,
    pub dt: f32,
}

impl Universe {
    pub fn new(
        time:f32,
        animation_length: f32,
        body_start_positions: Vec<Body>,
        max_distance: f32,
        light_speed: f32,
        gravity_strength: f32,
        dt: f32,
    ) -> Universe {
        let mut bodies_path: Vec<Vec<Body>> = vec![body_start_positions.clone()];

        let mut current_bodies = body_start_positions.clone();
        for body in &mut current_bodies {
            body.vel = -body.vel
        }
        for _ in 0..(max_distance / dt / light_speed).ceil() as usize {
            for a in 0..current_bodies.len() {
                for b in 0..current_bodies.len() {
                    if a != b {
                        if current_bodies[b].mass != 0.0 {
                            let vel = (current_bodies[b].pos - current_bodies[a].pos).normalize()
                                * (gravity_strength * current_bodies[b].mass
                                    / (current_bodies[b].pos - current_bodies[a].pos).magnitude2());
                            current_bodies[a].vel += vel * dt;
                        }
                    }
                }
                let vel = current_bodies[a].vel * dt;
                current_bodies[a].pos += vel;
            }
            let mut new_vec:Vec<Vec<Body>> = vec![current_bodies.clone()];
            new_vec.append(&mut bodies_path.clone());
            bodies_path = new_vec.clone();
        }

        let mut current_bodies = body_start_positions.clone();

        for _ in 0..(animation_length / dt) as usize {
            for a in 0..current_bodies.len() {
                for b in 0..current_bodies.len() {
                    if a != b {
                        if current_bodies[b].mass != 0.0 {
                            let vel = (current_bodies[b].pos - current_bodies[a].pos).normalize()
                                * (gravity_strength * current_bodies[b].mass
                                    / (current_bodies[b].pos - current_bodies[a].pos).magnitude2());
                            current_bodies[a].vel += vel * dt;
                        }
                    }
                }
                let vel = current_bodies[a].vel * dt;
                current_bodies[a].pos += vel;
            }
            bodies_path.append(&mut vec![current_bodies.clone()]);
        }

        return Universe {
            time: time,
            animation_length,
            bodies_path,
            max_distance,
            light_speed,
            gravity_strength,
            dt,
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
        &self.bodies_path[((self.bodies_path.len() as f32 * time) as usize).clamp(0, self.bodies_path.len() - 1)]
    }
}

fn trace_ray(x: f32, y: f32, aspect: f32, universe: &Universe) -> ColorF32 {
    let mut photon_pos = vec3(0.0, 0.0, -10.0);
    let mut photon_dir =
        vec3((x * 2.0 - 1.0) * aspect, y * 2.0 - 1.0, 1.0).normalize_to(universe.light_speed);

    let mut elapsed = 0.0;
    for _ in 0..universe.light_iter_count() {
        elapsed -= universe.dt;
        let time = universe.time + elapsed;

        for body in universe.get_bodies_at_time_percent(universe.time_percent(time)) {
            if photon_pos.distance2(body.pos) < body.radius * body.radius {
                return body.color;
            }

            if body.mass != 0.0 {
                let gravity =
                    (universe.gravity_strength * body.mass) / body.pos.distance2(photon_pos);

                // black hole pull is greater than light speed, the light cannot escape
                if gravity > universe.light_speed {
                    return ColorF32 {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                    };
                }

                photon_dir += (body.pos - photon_pos).normalize() * gravity * universe.dt;
            }
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

pub fn trace_rays(pixels: &mut [ColorF32], width: usize, height: usize, universe: &Universe) {
    let pixel_count = pixels.len();
    assert_eq!(pixel_count, width * height);
    let aspect = width as f32 / height as f32;

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
