use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::wgpu_state::WgpuState;

#[derive(Default)]
pub struct App<'a> {
    window: Option<Arc<Window>>,
    wgpu_state: Option<WgpuState<'a>>
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_title("WiW app");
            let window = Arc::new(
                event_loop.create_window(win_attr).unwrap());
            self.window = Some(window.clone());

            let state = WgpuState::new(window.clone());
            self.wgpu_state = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, 
                    window_id: WindowId, event: WindowEvent) {
        if self.window.as_ref().unwrap().id() != window_id { return; }
        if !self.wgpu_state.as_mut().unwrap().input(&event) {
            match event {
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                } => {
                    println!("The close button was pressed; stopping");
                    event_loop.exit();
                },
                WindowEvent::Resized(new_size) => {
                    if let (Some(wgpu_state), Some(window)) =
                        (self.wgpu_state.as_mut(), self.window.as_mut()) {
                        wgpu_state.resize((new_size.width, new_size.height));
                        window.request_redraw();
                    }
                },
                WindowEvent::RedrawRequested => {
                    self.window.as_ref().unwrap().request_redraw();
                    
                    self.wgpu_state.as_mut().unwrap().update();
                    if let Some(wgpu_state) = self.wgpu_state.as_mut() {
                        wgpu_state.render().expect("TODO: panic message");
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
