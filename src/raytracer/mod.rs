use wgpu::{Device, Queue, RenderPipeline, Sampler, StorageTextureAccess, Surface, SurfaceConfiguration, TextureDimension, TextureFormat, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use crate::Scene;

pub struct RayTracer {
    // scene_parameters: wgpu::Buffer,
    ray_tracing_pipeline: wgpu::ComputePipeline,
    ray_tracing_bind_group: wgpu::BindGroup,
    render_pipeline: RenderPipeline,
    render_pipeline_bind_group: wgpu::BindGroup,
}

impl RayTracer {
    pub fn new(device: &Device,
               _queue: &Queue,
               surface_config: &SurfaceConfiguration,
               _scene: &Scene,
               size: &PhysicalSize<u32>) -> Option<Self> {


        let (color_buffer_view, sampler) =
        create_color_buffer_assets(device, size);

        let (ray_tracing_bind_group, ray_tracing_pipeline) =
        create_ray_tracing_pipeline(device, &color_buffer_view);

        let (render_pipeline_bind_group, render_pipeline) =
            create_render_pipeline(device, surface_config.format, &color_buffer_view, &sampler);

        Some(Self {
            // scene_parameters: wgpu::Buffer,
            ray_tracing_pipeline,
            ray_tracing_bind_group,
            render_pipeline,
            render_pipeline_bind_group,
        })

    }

    pub fn resize(&mut self,
                  device: &Device,
                  surface: &mut Surface,
                  surface_config: &mut SurfaceConfiguration,
                  new_size: (u32, u32)) {

        let (width, height) = new_size;
        surface_config.width = width;
        surface_config.height = height;
        surface.configure(device, surface_config);
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

    pub fn update(&mut self) {}

    pub fn render(&mut self,
                  surface: &mut Surface,
                  device: & Device,
                  queue: & Queue, size: &PhysicalSize<u32>) -> Result<(), wgpu::SurfaceError> {

        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(
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
            ray_tracing_pass.dispatch_workgroups(size.width, size.height, 1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_pipeline_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
        queue.submit(Some(encoder.finish()));
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
        wgpu::include_wgsl!("../shaders/screen_shader.wgsl")
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
        wgpu::include_wgsl!("../shaders/raytracer_kernel.wgsl")
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