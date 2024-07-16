use std::sync::Arc;
use wgpu::{Device, RenderPipeline, Sampler, StorageTextureAccess, TextureDimension, TextureFormat, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct WgpuState<'a> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'a>,
    surface_config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    ray_tracing_pipeline: wgpu::ComputePipeline,
    ray_tracing_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    render_pipeline_bind_group: wgpu::BindGroup,
}

impl<'a> WgpuState<'a> {
    pub fn new(window: Arc<Window>) -> WgpuState<'a> {
        pollster::block_on(WgpuState::new_async(window))    
    }
    
    async fn new_async(window: Arc<Window>) -> WgpuState<'a> {
        let size = window.inner_size();
        
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            }
        );
        
        let surface = instance.create_surface(
            Arc::clone(&window)).unwrap();
        
        let adapter =  instance.request_adapter(
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
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        let clear_color = wgpu::Color::GREEN;

        let (color_buffer_view, sampler) =
            create_color_buffer_assets(&device, &size);

        let (ray_tracing_bind_group, ray_tracing_pipeline) =
            create_ray_tracing_pipeline(&device, &color_buffer_view);
        
        let (render_pipeline_bind_group, render_pipeline) =
            create_render_pipeline(&device,
                                   surface_config.format,
                                   &color_buffer_view,
                                   &sampler);
        
        Self {
            instance,
            surface,
            surface_config,
            adapter,
            device,
            queue,
            size,
            clear_color,
            ray_tracing_pipeline,
            ray_tracing_bind_group,
            render_pipeline,
            render_pipeline_bind_group
        }
    }
    
    pub fn resize(&mut self, new_size: (u32, u32)) {
        let (width, height) = new_size;
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }
    
    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        // match event {
        //     WindowEvent::CursorMoved { position, .. } => {
        //         self.clear_color = wgpu::Color {
        //             r: 0.0,
        //             g: 0.0,
        //             // r: position.x as f64 / self.size.width as f64,
        //             // g: position.y as f64 / self.size.height as f64,
        //             b: 0.0,
        //             a: 1.0,
        //         };
        //         true
        //     },
        //     _ => false
        // }
        false
    }
    
    pub fn update(&mut self) {
        
    }
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
          &wgpu::CommandEncoderDescriptor {
              label: Some("Render Encoder"),
          });

        {
            let mut ray_tracing_pass = encoder.begin_compute_pass(
                &wgpu::ComputePassDescriptor {
                    label: Some("Compute pass"),
                    timestamp_writes: None,
                }
            );
            ray_tracing_pass.set_pipeline(&self.ray_tracing_pipeline);
            ray_tracing_pass.set_bind_group(0, &self.ray_tracing_bind_group, &[]);
            ray_tracing_pass.dispatch_workgroups(self.size.width, self.size.height, 1);
        }
        
        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_pipeline_bind_group,&[]);
            render_pass.draw(0..6, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

}

fn create_color_buffer_assets(device: &Device, size: &PhysicalSize<u32>)
                              -> (wgpu::TextureView, wgpu::Sampler) {
    let texture_size = wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    };

    let color_buffer = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("color buffer texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST |
            wgpu::TextureUsages::STORAGE_BINDING |
            wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    
    let color_buffer_view = color_buffer.create_view(
        &wgpu::TextureViewDescriptor {
            label: None,
            format: None,
            dimension: None,
            aspect: Default::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        }
    );
    
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    (color_buffer_view, sampler)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    swap_chain_format: wgpu::TextureFormat,
    color_buffer_view: &TextureView,
    sampler: &Sampler)
    -> (wgpu::BindGroup, wgpu::RenderPipeline) {

    let render_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("render bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }
            ],
        }
    );

    let render_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: Some("render bind group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(color_buffer_view),
                }
            ],
        }
    );

    let shader = device.create_shader_module(
        wgpu::include_wgsl!("shaders/screen_shader.wgsl")
    );

    let render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[&render_bind_group_layout],
            push_constant_ranges: &[],
        });

    let rp = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs",
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: swap_chain_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState{
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    (render_bind_group, rp)
}

fn create_ray_tracing_pipeline(
    device: &wgpu::Device,
    color_buffer_view: &TextureView)
    -> (wgpu::BindGroup, wgpu::ComputePipeline) {

    let ray_tracing_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("ray tracing bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }
            ],
        }
    );
    let ray_tracing_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: Some("ray tracing bind group"),
            layout: &ray_tracing_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(color_buffer_view),
                }
            ],
        }
    );
    
    let ray_tracer_pipeline_layout = device.create_pipeline_layout(
        &wgpu::PipelineLayoutDescriptor {
            label: Some("ray tracer pipeline layout"),
            bind_group_layouts: &[&ray_tracing_bind_group_layout],
            push_constant_ranges: &[],
        }
    );

    let shader = device.create_shader_module(
        wgpu::include_wgsl!("shaders/raytracer_kernel.wgsl")
    );

    let ray_tracing_pipeline = device.create_compute_pipeline(
        &wgpu::ComputePipelineDescriptor {
            label: Some("ray tracing pipeline"),
            layout: Some(&ray_tracer_pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
        }
    );

    (ray_tracing_bind_group, ray_tracing_pipeline)
}
