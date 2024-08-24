use glam::{Vec3, Vec4, Vec4Swizzles};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Sphere {
    pub center: Vec4,
    // albedo: Vec3,
    radius: f32,
    material_idx: u32,
    _buffer: [u32; 2],
}

unsafe impl bytemuck::Pod for Sphere {}
unsafe impl bytemuck::Zeroable for Sphere {}


impl Sphere {
    pub fn new(center: Vec3, radius: f32, material_idx: u32) -> Self {
        Self { center: center.extend(0.0), radius, material_idx, _buffer: [0u32;2] }
    }

    pub fn get_aabb(&self) -> (Vec3, Vec3) {
        let aabb_min = self.center.xyz() - Vec3::splat(self.radius);
        let aabb_max = self.center.xyz() + Vec3::splat(self.radius);
        (aabb_min, aabb_max)
    }
}