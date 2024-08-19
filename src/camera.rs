use glam::{Vec3};

pub struct Camera {
    pub position: Vec3,
    pub forwards: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub vfov: f32,
    pub defocus_angle: f32,
    pub focus_distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        let lookAt = Vec3::new(0.0, 0.0, 0.0);
        let lookFrom = Vec3::new(13.0, 2.0, 3.0);
        // let lookAt = Vec3::new(0.0, 0.0, -1.0);
        // let lookFrom = Vec3::new(-2.0, 2.0, 1.0);
        let forwards = (lookAt - lookFrom).normalize();
        let right = forwards.cross(Vec3::new(0.0, 1.0, 0.0)).normalize();
        let up = right.cross(forwards);
        let vfov = 20.0f32;
        let defocus_angle = 0.6_f32;
        let focus_distance = 10.0_f32;
        // let defocus_angle = 0.0_f32;
        // let focus_distance = 3.4_f32;

        Self {
            position: lookFrom,
            forwards,
            right,
            up,
            vfov,
            defocus_angle,
            focus_distance
        }
    }
}

// pub struct CameraController {
//     vfov: f32
// }
//
// impl CameraController {
//     pub fn get_camera() -> Camera {
//
//     }
// }