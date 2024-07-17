use glam::Vec3;

pub struct Sphere {
    center: Vec3,
    radius: f32,
    albedo: Vec3,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, albedo: Vec3) -> Self {
        Self {center, radius, albedo}
    }
}