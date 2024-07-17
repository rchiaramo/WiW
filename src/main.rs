use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};
use wiw::{App, Scene, Camera};
use glam::Vec3;

fn main() -> Result<(), EventLoopError> {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let scene = Scene::new();
    let camera = Camera::new(Vec3::new(-20.0, 0.0, 0.0));
    // let camera = Camera::new(Vec3::new(0.75, 1.0, 0.25));
    let mut app = App::new(scene, camera);
    event_loop.run_app(&mut app)
}