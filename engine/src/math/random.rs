use super::Float;
use libc::{rand, srand, RAND_MAX};

// Todo: these functions use libc's srand and rand, which _isn't thread safe!_
pub fn seed_rand(seed: u32) {
    unsafe { srand(seed) };
}

pub fn rand_int(min: i32, max: i32) -> i32 {
    rand_float(min as f32, max as f32 + 1.) as i32
}

pub fn rand_float(min: f32, max: f32) -> f32 {
    min + unsafe { rand() } as f32 / RAND_MAX as f32 * (max - min)
}

pub fn pos_rand(n: f32) -> f32 {
    ((n * 12.9898).sin() * 43758.5453).fract()
}

pub fn noise(n: f32) -> f32 {
    let min_noise = pos_rand(n.floor());
    let max_noise = pos_rand(n.floor() + 1.);

    min_noise + (max_noise - min_noise) * n.fract()
}
