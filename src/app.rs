use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::{Camera, RayTracer};
use crate::gpu_structs::GPUFrameBuffer;
use crate::gpu_timing::QueryResults;

#[derive(Default)]
pub struct App<'a> {
    window: Option<Arc<Window>>,
    wgpu_state: Option<WgpuState<'a>>,
    renderer: Option<RayTracer>,
    render_parameters: RenderParameters,
    last_render_params: RenderParameters,
    render_progress: RenderProgress,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
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

            let max_viewport_resolution = window
                .available_monitors()
                .map(|monitor| -> u32 {
                    let viewport = monitor.size();
                    let size = (viewport.width, viewport.height);
                    size.0 * size.1
                })
                .max()
                .expect("must have at least one monitor");

            let size = {
                let viewport = window.inner_size();
                (viewport.width, viewport.height)
            };
            
            self.render_parameters.viewport_size = size;

            if let Some(state) = &self.wgpu_state {
                self.renderer = 
                    RayTracer::new(&state.device, max_viewport_resolution, size, &mut self.render_parameters);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, 
                    window_id: WindowId, event: WindowEvent) {
        let window = self.window.as_ref().unwrap();
        if window.id() != window_id { return; }

        let renderer = self.renderer.as_mut().unwrap();
        let state = self.wgpu_state.as_mut().unwrap();
        
        let done = self.render_progress.progress(5);
        let viewport_size = self.render_parameters.get_viewport();
        let frame = GPUFrameBuffer::new(
            viewport_size.0,
            viewport_size.1,
            self.render_progress.frame(),
            self.render_progress.accumulated_samples()
        );
        
        // if self.render_parameters != self.last_render_params {
        //     self.render_parameters.sampling_parameters.reset()
        // }
        // 
        // self.last_render_params = self.render_parameters;

        if !renderer.input(&event) {
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
                }

                WindowEvent::Resized(new_size) => {
                    let (width, height) = (new_size.width, new_size.height);
                    self.render_parameters.set_viewport((width, height));
                    state.resize((width, height));
                    
                    renderer.resize(&state.device,
                                    &state.queue,
                                    &self.render_parameters);
                    window.request_redraw();
                    
                }
                
                WindowEvent::CursorMoved { position, ..} => {
                    self.cursor_position = position;
                }

                WindowEvent::MouseInput { state, ..
                } => {
                    if state.is_pressed() {
                        println!("cursor position {:?}", self.cursor_position);
                    }
                }

                WindowEvent::RedrawRequested => {
                    if !done {
                        renderer.update(&state.queue, frame);
                        let queries =
                            renderer.render(
                                &mut state.surface,
                                &state.device,
                                &state.queue,
                                self.render_parameters.viewport_size
                            );
                        let raw_results = queries.wait_for_results(&state.device);
                        // println!("Raw timestamp buffer contents: {:?}", raw_results);
                        // QueryResults::from_raw_results(raw_results).print(&state.queue);
                    }
                }
                _ => {}
            }
        }
        self.window.as_ref().unwrap().request_redraw();
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
        ).await?;

        // Check timestamp features.
        let features = adapter.features()
            & (wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);
        // if features.contains(wgpu::Features::BGRA8UNORM_STORAGE)
        // {
        //     println!("Adapter has bgra8unorm storage");
        // } else {
        //     println!("Adapter does not have this storage");
        // }
        // if features.contains(wgpu::Features::TIMESTAMP_QUERY) {
        //     println!("Adapter supports timestamp queries.");
        // } else {
        //     println!("Adapter does not support timestamp queries, aborting.");
        // }
        // let timestamps_inside_passes = features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);
        // if timestamps_inside_passes {
        //     println!("Adapter supports timestamp queries within encoders.");
        // } else {
        //     println!("Adapter does not support timestamp queries within encoders.");
        // }

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: features, // wgpu::Features::empty(),
                required_limits: wgpu::Limits {
                    max_storage_buffer_binding_size: 512_u32 << 20,
                    ..Default::default()
                },
                label: None,
                memory_hints: Default::default(),
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
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.0,
            height: size.1,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        Some(Self {
            surface,
            surface_config,
            device,
            queue,
        })
    }
    
    fn resize(&mut self, new_size: (u32, u32))
    {
        self.surface_config.width = new_size.0;
        self.surface_config.height = new_size.1;
        self.surface.configure(&self.device, &self.surface_config);
    }
}

pub struct SamplingParameters {
    pub samples_per_frame: u32,
    pub num_bounces: u32,
    pub clear_image_buffer: u32,
}

impl Default for SamplingParameters {
    fn default() -> Self {

        Self {
            samples_per_frame: 5_u32,
            num_bounces: 50_u32,
            clear_image_buffer: 0_u32,
        }
    }
}

impl SamplingParameters {
    fn reset(&mut self) {
        self.clear_image_buffer = 1;
    }
    
    fn accumulate(&mut self) {
        self.clear_image_buffer = 0;
    }
}

#[derive(Default)]
pub struct RenderParameters {
    camera: Camera,
    sampling_parameters: SamplingParameters,
    viewport_size: (u32, u32),
    frame: u32,
    accumulated_samples: u32,
}

impl RenderParameters {
    // pub fn new_from(other: &Self) -> Self {
    //     Self {
    //         camera: other.camera,
    //         sampling_parameters: other.sampling_parameters,
    //         viewport_size: other.viewport_size
    //     }
    // }
    
    pub fn set_viewport(&mut self, size: (u32, u32)) {
        self.viewport_size = size;
    }
    
    pub fn get_viewport(&self) -> (u32, u32) {
        self.viewport_size
    }
    
    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    
    pub fn update_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }
}

pub struct RenderProgress {
    frame: u32,
    samples_per_pixel: u32,
    accumulated_samples: u32,
}

impl Default for RenderProgress {
    fn default() -> Self {
        Self {
            frame: 0,
            samples_per_pixel: 100,
            accumulated_samples: 0
        }
    }
}

impl RenderProgress {
    pub fn new(spp: u32) -> Self {
        Self {
            frame: 0,
            samples_per_pixel: spp,
            accumulated_samples: 0
        }
    }
    
    pub fn reset(&mut self) {
        self.frame = 0;
        self.accumulated_samples = 0;
    }
    
    pub fn frame(&self) -> u32 {
        self.frame
    }

    pub fn accumulated_samples(&self) -> u32 {
        self.accumulated_samples
    }
    
    pub fn progress(&mut self, spf: u32) -> bool {
        self.frame += 1;
        self.accumulated_samples += spf;
        self.accumulated_samples >= self.samples_per_pixel
    }
}



