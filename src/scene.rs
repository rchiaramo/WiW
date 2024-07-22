use glam::{Vec3};
use crate::material::Material;
use crate::Sphere;


pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub materials: Vec<Material>,
}

impl Scene {
    pub fn new() -> Self {
        let mat_ground = Material::Lambertian(Vec3::new(0.8, 0.8, 0.0));
        let mat_center = Material::Lambertian(Vec3::new(0.1, 0.2, 0.5));
        let mat_left = Material::Dielectric(1.00/1.33);
        let mat_bubble = Material::Dielectric(1.00/1.50);
        let mat_right = Material::Metal(Vec3::new(0.8, 0.6, 0.2), 1.0);

        let mut materials = vec![mat_ground, mat_center, mat_left, mat_right, mat_bubble];

        let ground = Sphere::new(
            Vec3::new(0.0, -100.5, -1.0),
            100.0,
            0);
        let center = Sphere::new(
            Vec3::new(0.0, 0.0, -1.2),
            0.5,
            1);
        let left = Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.5,
            2);
        let bubble = Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.4,
            4);
        let right = Sphere::new(
            Vec3::new(1.0, 0.0, -1.0),
            0.5,
            3);

        let mut spheres = vec![ground, center, left, right];

        Self { spheres, materials }
    }

}