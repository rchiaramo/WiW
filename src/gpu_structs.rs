use glam::{Vec3, Vec4};
use crate::app::SamplingParameters;
use crate::Camera;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GPUCamera {
    camera_position: Vec4,
    camera_forwards: Vec4,
    camera_right: Vec4,
    camera_up: Vec4,
    pixel_00: Vec4,
    du: Vec4,
    dv: Vec4,
    defocus_radius: f32,
    _buffer: [u32; 3]
}
unsafe impl bytemuck::Pod for GPUCamera {}
unsafe impl bytemuck::Zeroable for GPUCamera {}

impl GPUCamera {
    pub fn new(camera: &Camera, image_size: (u32, u32)) -> GPUCamera {
        let defocus_radius = camera.focus_distance * (0.5 * camera.defocus_angle).to_radians().tan();
        let theta = camera.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height: f32 = 2.0 * h * camera.focus_distance;
        let viewport_width: f32 = viewport_height * (image_size.0 as f32 / image_size.1 as f32);

        let viewport_u = viewport_width * camera.right;
        let viewport_v = -viewport_height * camera.up;

        let du = viewport_u / image_size.0 as f32;
        let dv = viewport_v / image_size.1 as f32;

        let upper_left = camera.position + camera.focus_distance * camera.forwards -
            0.5 * (viewport_u + viewport_v);
        let pixel_00 = upper_left + 0.5 * (du + dv);

        GPUCamera {
            camera_position: camera.position.extend(0.0),
            camera_forwards: camera.forwards.extend(0.0),
            camera_right: camera.right.extend(0.0),
            camera_up: camera.up.extend(0.0),
            pixel_00: pixel_00.extend(0.0),
            du: du.extend(0.0),
            dv: dv.extend(0.0),
            defocus_radius,
            _buffer: [0u32; 3]
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GPUSamplingParameters {
    samples_per_pixel: u32,
    num_bounces: u32,
    samples_per_frame: u32,
    total_samples_completed: u32,
    frame: u32,
    _buffer: [u32; 3],
}

// right now this is silly, but later when we add fields to this struct,
// we may have to do some padding for GPU
pub fn get_gpu_sampling_params(sampling_parameters: &SamplingParameters)
                           -> GPUSamplingParameters
{
    GPUSamplingParameters {
        samples_per_pixel: sampling_parameters.samples_per_pixel,
        num_bounces: sampling_parameters.num_bounces,
        samples_per_frame: sampling_parameters.samples_per_frame,
        total_samples_completed: sampling_parameters.total_samples_completed,
        frame: sampling_parameters.frame,
        _buffer: [0u32; 3]
    }
}