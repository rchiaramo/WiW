use glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SceneParameters {
    camera_position: Vec4,
    camera_forwards: Vec4,
    camera_right: Vec4,
    camera_up: Vec3,
    sphere_count: f32,
}

unsafe impl bytemuck::Pod for SceneParameters {}
unsafe impl bytemuck::Zeroable for SceneParameters {}

pub struct Camera {
    position: Vec3,
    theta: f32,
    phi: f32,
    forwards: Vec3,
    right: Vec3,
    up: Vec3,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            theta: 0.0,
            phi: 0.0,
            forwards: Vec3::new(1.0, 0.0, 0.0),
            right: Vec3::new(0.0, -1.0, 0.0),
            up: Vec3::new(0.0, 0.0, 1.0),
        }
    }

    fn recalculate_vectors(&mut self) {
        self.forwards = Vec3::new(
          self.theta.to_radians().cos() * self.phi.to_radians().cos(),
          self.theta.to_radians().sin() * self.phi.to_radians().cos(),
          self.phi.to_radians().sin()
        );
        self.right = self.forwards.cross(Vec3::new(0.0, 0.0, 1.0));
        self.up = self.right.cross(self.forwards);
    }

    pub fn get_scene_parameters(&self) -> SceneParameters {
        SceneParameters {
            camera_position: self.position.extend(0.0),
            camera_forwards: self.forwards.extend(0.0),
            camera_right: self.right.extend(0.0),
            camera_up: self.up,
            sphere_count: 32.0,
        }
    }
}