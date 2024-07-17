use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};
use wiw::{App, Scene};

fn main() -> Result<(), EventLoopError> {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let scene = Scene::new();
    let mut app = App::new(scene);
    event_loop.run_app(&mut app)
}