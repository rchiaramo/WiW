use glam::Vec3;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    center: Vec3,
    radius: f32,
    albedo: Vec3,
}

unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}


impl Sphere {
    pub fn new(center: Vec3, radius: f32, albedo: Vec3) -> Self {
        Self {center, radius, albedo}
    }
}