use glam::Vec3;
use crate::Sphere;


pub struct Scene {
    pub spheres: Vec<Sphere>,
}

impl Scene {
    pub fn new() -> Self {
        let sphere1 = Sphere::new(
            Vec3::new(0.0, 0.0, -1.0),
            0.5,
            Vec3::new(0.1, 0.2, 0.5));
        let sphere2 = Sphere::new(
            Vec3::new(0.0, -100.5, -1.0),
            100.0,
            Vec3::new(0.8, 0.8, 0.0));
        let mut spheres = vec![sphere1, sphere2];

        Self { spheres }
    }

}