use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::{Camera, RayTracer};
use crate::scene::Scene;


pub struct App<'a> {
    window: Option<Arc<Window>>,
    wgpu_state: Option<WgpuState<'a>>,
    renderer: Option<RayTracer>,
    scene: Scene,
    render_parameters: RenderParameters,
}

impl Default for App<'_> {
    fn default() -> Self {
        let scene = Scene::new();
        let camera = Camera::default();
        let render_parameters = RenderParameters { camera, viewport:(0, 0) };
        Self {window: None, wgpu_state: None, renderer: None, scene, render_parameters }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes()
                .with_inner_size(winit::dpi::PhysicalSize::new(1600, 900))
                .with_title("WiW app");
            let window = Arc::new(
                event_loop.create_window(win_attr).unwrap());
            self.window = Some(window.clone());

            self.wgpu_state = WgpuState::new(window.clone());

            let mut size = {
                let viewport = window.inner_size();
                (viewport.width, viewport.height)
            };
            self.render_parameters.viewport = size;

            // This code properly gets the resolution of the largest window, but when passed to
            // the renderer to use as the biggest array value, clips the image for some reason

            // let _max_viewport_resolution = window
            //     .available_monitors()
            //     .map(|monitor| -> u32 {
            //         let viewport = monitor.size();
            //         size = (viewport.width, viewport.height);
            //         size.0 * size.1
            //     })
            //     .max()
            //     .expect("must have at least one monitor");

            if let Some(state) = &self.wgpu_state {
                self.renderer = RayTracer::new(
                    &state.device,
                    &state.queue,
                    &state.surface_config,
                    &self.render_parameters,
                    &self.scene,
                    self.render_parameters.viewport
                );
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, 
                    window_id: WindowId, event: WindowEvent) {
        if self.window.as_ref().unwrap().id() != window_id { return; }
        if !self.renderer.as_mut().unwrap().input(&event) {
            match event {
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                } => {
                    event_loop.exit();
                },
                WindowEvent::Resized(new_size) => {
                    if let (Some(renderer), Some(state)) =
                        (self.renderer.as_mut(), self.wgpu_state.as_mut()) {
                        self.render_parameters.viewport = (new_size.width, new_size.height);
                        renderer.resize(&state.device,
                                        &state.queue,
                                        &mut state.surface,
                                        &mut state.surface_config,
                                        &self.render_parameters);
                        self.window.as_ref().unwrap().request_redraw();
                    }
                },
                WindowEvent::RedrawRequested => {
                    self.window.as_ref().unwrap().request_redraw();

                    if let (Some(renderer), Some(state)) =
                        (self.renderer.as_mut(), self.wgpu_state.as_mut()) {
                            renderer.update();
                            renderer.render(
                                &mut state.surface,
                                &state.device,
                                &state.queue,
                                self.render_parameters.viewport
                            ).expect("TODO: panic message");
                    }
                }
                _ => {}
            }
        }
    }
}

pub struct WgpuState<'a> {
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'a> WgpuState<'a> {
    pub fn new(window: Arc<Window>) -> Option<WgpuState<'a>> {
        pollster::block_on(WgpuState::new_async(window))
    }

    async fn new_async(window: Arc<Window>) -> Option<WgpuState<'a>> {
        let size = {
            let viewport = window.inner_size();
            (viewport.width, viewport.height)
        };

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            }
        );

        let surface = instance.create_surface(
            Arc::clone(&window)).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        // I need to figure out why Bgra8Unorm looks best

        // let surface_format = surface_capabilities.formats.iter()
        //     .find(|f| f.is_srgb())
        //     .copied()
        //     .unwrap_or(surface_capabilities.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm, // surface_format,
            width: size.0,
            height: size.1,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Some(Self {
            surface,
            surface_config,
            device,
            queue,
        })
    }
}

pub struct RenderParameters {
    pub camera: Camera,
    pub viewport: (u32, u32)
}
