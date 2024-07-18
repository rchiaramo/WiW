use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    center: Vec4,
    albedo: Vec3,
    radius: f32,
}

unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}


impl Sphere {
    pub fn new(center: Vec3, radius: f32, albedo: Vec3) -> Self {
        Self { center: center.extend(0.0), albedo, radius }
    }
}