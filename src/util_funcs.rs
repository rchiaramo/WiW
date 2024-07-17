use rand::Rng;
use rand::seq::SliceRandom;
use glam::Vec3;

pub fn reflect(incident: Vec3, norm: Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(norm) * norm
}

#[allow(dead_code)]
pub fn random_u32() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen::<u32>()
}

#[allow(dead_code)]
pub fn random_f32() -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>()
}

#[allow(dead_code)]
pub fn random_range_f32(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min .. max)
}

pub fn random_vec3_range(min: f32, max: f32) -> Vec3 {
    Vec3::new(random_range_f32(min, max),
              random_range_f32(min, max),
              random_range_f32(min, max))
}

#[allow(dead_code)]
pub fn shuffle_array<T>(mut a: Vec<T>) -> Vec<T> {
    let mut rng = rand::thread_rng();
    a.shuffle(&mut rng);
    a
}