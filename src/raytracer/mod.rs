use glam::{Vec4};
use wgpu::{BindGroupDescriptor, BindGroupEntry,
           BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
           Buffer, BufferBindingType, BufferUsages, Device, Queue,
           RenderPipeline, ShaderStages, StorageTextureAccess,
           Surface, SurfaceConfiguration, TextureDimension,
           TextureFormat, TextureView, TextureViewDimension};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::event::WindowEvent;
use crate::app::{RenderParameters, SamplingParameters};
use crate::{Camera, Scene};

pub struct RayTracer {
    camera_buffer: Buffer,
    sampling_parameters_buffer: Buffer,
    image_bind_group: wgpu::BindGroup,
    scene_bind_group: wgpu::BindGroup,
    parameters_bind_group: wgpu::BindGroup,
    ray_tracer_pipeline: wgpu::ComputePipeline,
    display_pipeline_bind_group: wgpu::BindGroup,
    display_pipeline: RenderPipeline,
}

impl RayTracer {
    pub fn new(device: &Device,
               queue: &Queue,
               surface_config: &SurfaceConfiguration,
               render_parameters: &RenderParameters,
               scene: &Scene,
               max_image_size: (u32, u32)) -> Option<Self> {

        // create the image_buffer that the compute shader will use to store image
        let (image_bind_group,
            image_bind_group_layout,
            image_buffer_view) = create_image_buffer(device, max_image_size);

        // create the scene bind group that holds objects and materials
        let (scene_bind_group, scene_bind_group_layout)
            = create_scene_bind_group(device, scene);

        // create the parameters bind group to interact with GPU during runtime
        let (parameters_bind_group,
            parameter_bind_group_layout,
            camera_buffer,
            sampling_parameters_buffer)
            = create_parameters_bind_group(device, queue, render_parameters);

        let ray_tracer_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("ray tracer pipeline layout"),
                bind_group_layouts: &[
                    &image_bind_group_layout,
                    &scene_bind_group_layout,
                    &parameter_bind_group_layout
                ],
                push_constant_ranges: &[],
            }
        );

        let shader = device.create_shader_module(
            wgpu::include_wgsl!("../shaders/raytracer_kernel.wgsl")
        );

        let ray_tracer_pipeline = device.create_compute_pipeline(
            &wgpu::ComputePipelineDescriptor {
                label: Some("ray tracer pipeline"),
                layout: Some(&ray_tracer_pipeline_layout),
                module: &shader,
                entry_point: "main",
                compilation_options: Default::default(),
                cache: None,
            }
        );

        let (display_pipeline_bind_group, display_pipeline) =
            create_display_pipeline(device, surface_config.format, &image_buffer_view);

        Some(Self {
            camera_buffer,
            sampling_parameters_buffer,
            image_bind_group,
            scene_bind_group,
            parameters_bind_group,
            ray_tracer_pipeline,
            display_pipeline,
            display_pipeline_bind_group,
        })

    }

    pub fn resize(&mut self,
                  device: &Device,
                  queue: &Queue,
                  surface: &mut Surface,
                  surface_config: &mut SurfaceConfiguration,
                  render_parameters: &RenderParameters) {
        let (width, height) = render_parameters.viewport;
        surface_config.width = width;
        surface_config.height = height;
        surface.configure(device, surface_config);

        let scene_parameters = get_gpu_camera(&render_parameters.camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[scene_parameters]));
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
                  queue: & Queue,
                  size: (u32, u32)
    ) -> Result<(), wgpu::SurfaceError> {

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
            ray_tracing_pass.set_pipeline(&self.ray_tracer_pipeline);
            ray_tracing_pass.set_bind_group(0, &self.image_bind_group, &[]);
            ray_tracing_pass.set_bind_group(1, &self.scene_bind_group, &[]);
            ray_tracing_pass.set_bind_group(2, &self.parameters_bind_group, &[]);
            ray_tracing_pass.dispatch_workgroups(size.0, size.1, 1);
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
            render_pass.set_pipeline(&self.display_pipeline);
            render_pass.set_bind_group(0, &self.display_pipeline_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }
}

fn create_image_buffer(device: &Device, max_image_size: (u32, u32))
                              -> (wgpu::BindGroup, wgpu::BindGroupLayout, wgpu::TextureView) {
    let texture_size = wgpu::Extent3d {
        width: max_image_size.0,
        height: max_image_size.1,
        depth_or_array_layers: 1,
    };

    let image_buffer = device.create_texture(&wgpu::TextureDescriptor {
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

    let image_buffer_view = image_buffer.create_view(
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

    let image_bind_group_layout = device.create_bind_group_layout(
        &wgpu::BindGroupLayoutDescriptor {
            label: Some("image bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }
            ]
        }
    );

    let image_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: Some("image bind group"),
            layout: &image_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image_buffer_view),
                }
            ],
        }
    );
    (image_bind_group, image_bind_group_layout, image_buffer_view)
}

fn create_scene_bind_group(device: &Device, scene: &Scene)
    -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let spheres = &scene.spheres;
    let sphere_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Sphere storage buffer"),
        contents: bytemuck::cast_slice(spheres.as_slice()),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    let materials = &scene.materials;
    let materials_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Materials storage buffer"),
        contents: bytemuck::cast_slice(materials.as_slice()),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    });

    let scene_bind_group_layout = device.create_bind_group_layout(
        &BindGroupLayoutDescriptor {
            label: Some("scene bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        }
    );
    let scene_bind_group = device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: Some("scene bind group"),
            layout: &scene_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sphere_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: materials_buffer.as_entire_binding(),
                }
            ],
        }
    );
    (scene_bind_group, scene_bind_group_layout)
}

fn create_parameters_bind_group(device: &Device,
                                queue: &Queue,
                                render_parameters: &RenderParameters)
    -> (wgpu::BindGroup, wgpu::BindGroupLayout, Buffer, Buffer) {
    // initialize the camera buffer
    let camera_desc = wgpu::BufferDescriptor {
        label: Some("camera uniform buffer"),
        size: 64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };
    let camera = get_gpu_camera(&render_parameters.camera);
    let camera_buffer = device.create_buffer(&camera_desc);
    queue.write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera]));

    // initialize the sampling_parameters buffer
    let sampling_param_desc = wgpu::BufferDescriptor {
        label: Some("sampling parameters uniform buffer"),
        size: 8,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    };

    let sampling_parameters = get_gpu_sampling_params(&render_parameters.sampling_parameters);
    let sampling_parameters_buffer = device.create_buffer(&sampling_param_desc);
    queue.write_buffer(&sampling_parameters_buffer, 0, bytemuck::cast_slice(&[sampling_parameters]));

    let parameters_bind_group_layout = device.create_bind_group_layout(
        &BindGroupLayoutDescriptor {
            label: Some("parameters bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        }
    );

    let parameters_bind_group = device.create_bind_group(
        &BindGroupDescriptor {
            label: Some("parameters bind group"),
            layout: &parameters_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: sampling_parameters_buffer.as_entire_binding(),
                }
            ],
        }
    );

    (parameters_bind_group, parameters_bind_group_layout, camera_buffer, sampling_parameters_buffer)
}

fn create_display_pipeline(
    device: &wgpu::Device,
    surface_config_format: wgpu::TextureFormat,
    image_buffer_view: &TextureView)
    -> (wgpu::BindGroup, wgpu::RenderPipeline) {

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

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
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(image_buffer_view),
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
                format: surface_config_format,
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
        cache: None,
    });

    (render_bind_group, rp)
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GPUCamera {
    camera_position: Vec4,
    camera_forwards: Vec4,
    camera_right: Vec4,
    camera_up: Vec4,
}
unsafe impl bytemuck::Pod for GPUCamera {}
unsafe impl bytemuck::Zeroable for GPUCamera {}

fn get_gpu_camera(camera: &Camera) -> GPUCamera {
    GPUCamera {
        camera_position: camera.position.extend(0.0),
        camera_forwards: camera.forwards.extend(0.0),
        camera_right: camera.right.extend(0.0),
        camera_up: camera.up.extend(0.0),
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUSamplingParameters {
    samples_per_pixel: u32,
    num_bounces: u32,
}

// right now this is silly, but later when we add fields to this struct,
// we may have to do some padding for GPU
fn get_gpu_sampling_params(sampling_parameters: &SamplingParameters)
    -> GPUSamplingParameters
{
    GPUSamplingParameters {
        samples_per_pixel: sampling_parameters.samples_per_pixel,
        num_bounces: sampling_parameters.num_bounces,
    }
}