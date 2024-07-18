use glam::Vec3;
use crate::{Sphere, SPHERE_COUNT};
use crate::util_funcs::random_f32;


pub struct Scene {
    pub spheres: Vec<Sphere>,
}

impl Scene {
    pub fn new() -> Self {
        let mut spheres = Vec::<Sphere>::with_capacity(SPHERE_COUNT);
        for _i in 0..SPHERE_COUNT {
            let center = Vec3::new(
                3.0 + 7.0 * random_f32(),
                -5.0 + 10.0 * random_f32(),
                -5.0 + 10.0 * random_f32());
            let radius = 0.1 + 1.9 * random_f32();
            let albedo = Vec3::new(
                0.3 + 0.7 * random_f32(),
                0.3 + 0.7 * random_f32(),
                0.3 + 0.7 * random_f32());
            spheres.push(Sphere::new(center, radius, albedo))
        }

        Self { spheres }
    }

}