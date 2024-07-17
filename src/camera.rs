use glam::Vec3;

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
}