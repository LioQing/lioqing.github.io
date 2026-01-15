use glam::*;

use crate::{
    delta_time::{self, DeltaTime},
    frame::FrameMetadata,
    grid::{GRID_CELL_SIZE, GridMetadata, GridState, target::Target},
};

#[derive(Debug)]
pub struct GridProcessor {
    workgroup_size: u32,
    compute_pipeline: wgpu::ComputePipeline,
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    target: Target,
}

impl GridProcessor {
    pub fn new(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        grid_metadata: &GridMetadata,
        delta_time: &DeltaTime,
        grid_state: &GridState,
        mouse: Vec2,
    ) -> Self {
        let workgroup_size = {
            let mut max_size = None;

            // TODO: make a WorkgroupSolver to optimize this properly
            for i in (2..=8).rev() {
                let size = 1 << i;
                if device.limits().max_compute_workgroup_size_x >= size
                    && device.limits().max_compute_invocations_per_workgroup >= size
                {
                    max_size = Some(size);
                    break;
                }
            }

            log::debug!("Chosen workgroup size for GridProcessor: {max_size:?}");

            max_size.expect("device must support workgroup size of at least 4")
        };

        let target = Target::new(device, mouse - frame_metadata.top_left().as_vec2());

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Processor Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/grid_process.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid Processor Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Processor Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: grid_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: target.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: delta_time.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: grid_state.pos_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: grid_state.vel_buffer().as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grid Processor Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Grid Processor Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[
                    ("cell_size", GRID_CELL_SIZE as f64),
                    ("workgroup_size", workgroup_size as f64),
                ],
                ..Default::default()
            },
            cache: None,
        });

        Self {
            workgroup_size,
            compute_pipeline,
            bind_group_layout,
            bind_group,
            target,
        }
    }

    pub fn recreate_bind_group(
        &mut self,
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        grid_metadata: &GridMetadata,
        delta_time: &DeltaTime,
        grid_state: &GridState,
    ) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Processor Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: grid_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.target.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: delta_time.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: grid_state.pos_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: grid_state.vel_buffer().as_entire_binding(),
                },
            ],
        });
    }

    pub fn update_target(
        &mut self,
        queue: &wgpu::Queue,
        frame_metadata: &FrameMetadata,
        mouse: Vec2,
        delta_time: f32,
    ) {
        self.target.update(
            queue,
            mouse - frame_metadata.top_left().as_vec2(),
            delta_time,
        );
    }

    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, grid_resolution: UVec2) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Grid Processor Compute Pass"),
            ..Default::default()
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        compute_pass.dispatch_workgroups(
            (grid_resolution.x * grid_resolution.y * 2).div_ceil(self.workgroup_size),
            1,
            1,
        );
    }
}

#[derive(Debug)]
pub struct GridRenderer {
    render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl GridRenderer {
    pub fn new(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        grid_metadata: &GridMetadata,
        grid_state: &GridState,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let render_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Renderer Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/grid.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Grid Renderer Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Renderer Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: grid_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: grid_state.pos_buffer().as_entire_binding(),
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Grid Renderer Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compilation_options = wgpu::PipelineCompilationOptions {
            constants: &[("cell_size", GRID_CELL_SIZE as f64)],
            ..Default::default()
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Renderer Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader_module,
                entry_point: Some("vert_main"),
                buffers: &[],
                compilation_options: compilation_options.clone(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader_module,
                entry_point: Some("frag_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn recreate_bind_group(
        &mut self,
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        grid_metadata: &GridMetadata,
        grid_state: &GridState,
    ) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Renderer Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: grid_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: grid_state.pos_buffer().as_entire_binding(),
                },
            ],
        });
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        grid_resolution: UVec2,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Grid Renderer Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..4, 0..grid_resolution.x * grid_resolution.y * 2);
    }
}
