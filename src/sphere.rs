use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    center: Vec4,
    // albedo: Vec3,
    radius: f32,
    material_idx: u32,
    // _buffer: f32,
    // _buffer2: f32,
}

unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}


impl Sphere {
    pub fn new(center: Vec3, radius: f32, material_idx: u32) -> Self {
        Self { center: center.extend(0.0), radius, material_idx } //, _buffer: 0.0, _buffer2: 0.0 }
    }
}