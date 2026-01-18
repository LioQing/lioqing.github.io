use glam::*;
use wgpu::util::DeviceExt as _;

use crate::{
    frame::FrameMetadata,
    mar_sq::{
        quad::Quads,
        traits::{
            MarchingSquaresShape, MarchingSquaresShapeBuffer as _,
            MarchingSquaresShapeIndirect as _,
        },
    },
    meta_field::{self, MetaField},
    pipeline::{FADE_DIST, HEIGHT, MetaFieldMag, MetaFieldRenderer, RADIUS},
};

#[derive(Debug)]
pub struct MarchingSquaresProcessor<Shape: MarchingSquaresShape> {
    workgroup_size: u32,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    indirect_buffer: Shape::Indirect,
}

impl<Shape: MarchingSquaresShape> MarchingSquaresProcessor<Shape> {
    pub fn new(device: &wgpu::Device, meta_field: &MetaField, shape: &Shape::Buffer) -> Self {
        let workgroup_size = {
            let mut max_size = None;

            for i in (2..=8).rev() {
                let size = 1 << i;
                if device.limits().max_compute_workgroup_size_x >= size
                    && device.limits().max_compute_invocations_per_workgroup >= size
                {
                    max_size = Some(size);
                    break;
                }
            }

            log::debug!("Chosen workgroup size for MarchingSquaresProcessor: {max_size:?}");

            max_size.expect("device must support workgroup size of at least 4")
        };

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Marching Squares Processor Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Shape::PREPROCESS_SHADER.into()),
        });

        let indirect_buffer = Shape::Indirect::new(device);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Marching Squares Processor Bind Group Layout"),
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
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::ReadOnly,
                        format: meta_field.texture().format(),
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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
            label: Some("Marching Squares Processor Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: meta_field.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &meta_field
                            .texture()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: indirect_buffer.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: shape.buffer().as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Marching Squares Processor Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Marching Squares Processor Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[("workgroup_size", workgroup_size as f64)],
                ..Default::default()
            },
            cache: None,
        });

        Self {
            workgroup_size,
            compute_pipeline,
            bind_group_layout,
            bind_group,
            indirect_buffer,
        }
    }

    pub fn recreate_bind_group(
        &mut self,
        device: &wgpu::Device,
        meta_field: &MetaField,
        shape: &Shape::Buffer,
    ) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Marching Squares Processor Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: meta_field.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &meta_field
                            .texture()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.indirect_buffer.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: shape.buffer().as_entire_binding(),
                },
            ],
        });
    }

    pub fn process(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        meta_field_resolution: UVec2,
    ) {
        self.indirect_buffer.reset(queue);

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Marching Squares Processor Compute Pass"),
            ..Default::default()
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        compute_pass.dispatch_workgroups(
            (meta_field_resolution.x * meta_field_resolution.y).div_ceil(self.workgroup_size),
            1,
            1,
        );
    }
}

#[derive(Debug)]
pub struct MarchingSquaresShapeRenderer<Shape: MarchingSquaresShape> {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    phantom: std::marker::PhantomData<Shape>,
}

impl<Shape: MarchingSquaresShape> MarchingSquaresShapeRenderer<Shape> {
    pub fn new(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        shape: &Shape::Buffer,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let render_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Marching Squares Shape Renderer Shader Module"),
            source: wgpu::ShaderSource::Wgsl(Shape::RENDER_SHADER.into()),
        });

        let bind_group_layout = Self::create_bind_group_layout(
            device,
            Some("Marching Squares Shape Renderer Bind Group Layout"),
        );

        let bind_group = Self::create_bind_group(
            device,
            Some("Marching Squares Shape Renderer Bind Group"),
            &bind_group_layout,
            frame_metadata,
            shape,
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Marching Squares Shape Renderer Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Marching Squares Shape Renderer Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader_module,
                entry_point: Some("vert_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader_module,
                entry_point: Some("frag_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
            phantom: std::marker::PhantomData,
        }
    }

    pub fn recreate_bind_group(
        &mut self,
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        shape: &Shape::Buffer,
    ) {
        self.bind_group = Self::create_bind_group(
            device,
            Some("Marching Squares Shape Renderer Bind Group"),
            &self.bind_group_layout,
            frame_metadata,
            shape,
        );
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        marching_squares_processor: &MarchingSquaresProcessor<Shape>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Marching Squares Shape Renderer Render Pass"),
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
        render_pass.draw_indirect(marching_squares_processor.indirect_buffer.buffer(), 0);
    }

    pub fn create_bind_group_layout(
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn create_bind_group(
        device: &wgpu::Device,
        label: Option<&str>,
        bind_group_layout: &wgpu::BindGroupLayout,
        frame_metadata: &FrameMetadata,
        shape: &Shape::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: shape.buffer().as_entire_binding(),
                },
            ],
        })
    }
}

#[derive(Debug)]
pub struct MarchingSquaresLiquidQuadRenderer {
    render_pipeline: wgpu::RenderPipeline,
    quads_bind_group_layout: wgpu::BindGroupLayout,
    quads_bind_group: wgpu::BindGroup,
    meta_field_bind_group_layout: wgpu::BindGroupLayout,
    meta_field_bind_group: wgpu::BindGroup,
    background_bind_group_layout: wgpu::BindGroupLayout,
    background_bind_group: wgpu::BindGroup,
    background_color: wgpu::Buffer,
}

impl MarchingSquaresLiquidQuadRenderer {
    pub fn new(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        quads: &Quads,
        meta_field: &MetaField,
        background_view: &wgpu::TextureView,
        background_color: Vec3,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let background_color = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Marching Squares Liquid Quad Renderer Background Color Buffer"),
            contents: bytemuck::bytes_of(&background_color),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let render_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Marching Squares Liquid Quad Renderer Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/liquid_quad.wgsl").into()),
        });

        let quads_bind_group_layout =
            MarchingSquaresShapeRenderer::<Quads>::create_bind_group_layout(
                device,
                Some("Marching Squares Liquid Quad Renderer Quads Bind Group Layout"),
            );

        let quads_bind_group = MarchingSquaresShapeRenderer::<Quads>::create_bind_group(
            device,
            Some("Marching Squares Liquid Quad Renderer Quads Bind Group"),
            &quads_bind_group_layout,
            frame_metadata,
            quads,
        );

        let meta_field_bind_group_layout =
            MetaFieldRenderer::<MetaFieldMag>::create_bind_group_layout(
                device,
                Some("Marching Squares Liquid Quad Renderer Meta Field Bind Group Layout"),
            );

        let meta_field_bind_group = MetaFieldRenderer::<MetaFieldMag>::create_bind_group(
            device,
            Some("Marching Squares Liquid Quad Renderer Meta Field Bind Group"),
            &meta_field_bind_group_layout,
            meta_field,
        );

        let background_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Marching Squares Liquid Quad Renderer Background Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let background_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Marching Squares Liquid Quad Renderer Background Bind Group"),
            layout: &background_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(background_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            label: Some("Background Sampler"),
                            mag_filter: wgpu::FilterMode::Linear,
                            min_filter: wgpu::FilterMode::Linear,
                            mipmap_filter: wgpu::FilterMode::Linear,
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        background_color.as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Marching Squares Liquid Quad Renderer Pipeline Layout"),
                bind_group_layouts: &[
                    &quads_bind_group_layout,
                    &meta_field_bind_group_layout,
                    &background_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let compilation_options = wgpu::PipelineCompilationOptions {
            constants: &[
                ("radius", RADIUS),
                ("fade_dist", FADE_DIST),
                ("height", HEIGHT),
            ],
            ..Default::default()
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Marching Squares Liquid Quad Renderer Render Pipeline"),
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
                    blend: Some(wgpu::BlendState::REPLACE),
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
            quads_bind_group_layout,
            quads_bind_group,
            meta_field_bind_group_layout,
            meta_field_bind_group,
            background_bind_group_layout,
            background_bind_group,
            background_color,
        }
    }

    pub fn recreate_bind_group(
        &mut self,
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        quads: &Quads,
        meta_field: &MetaField,
        background_view: &wgpu::TextureView,
    ) {
        self.quads_bind_group = MarchingSquaresShapeRenderer::<Quads>::create_bind_group(
            device,
            Some("Marching Squares Liquid Quad Renderer Quads Bind Group"),
            &self.quads_bind_group_layout,
            frame_metadata,
            quads,
        );

        self.meta_field_bind_group = MetaFieldRenderer::<MetaFieldMag>::create_bind_group(
            device,
            Some("Marching Squares Liquid Quad Renderer Meta Field Bind Group"),
            &self.meta_field_bind_group_layout,
            meta_field,
        );

        self.background_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Marching Squares Liquid Quad Renderer Background Bind Group"),
            layout: &self.background_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(background_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            label: Some("Background Sampler"),
                            mag_filter: wgpu::FilterMode::Linear,
                            min_filter: wgpu::FilterMode::Linear,
                            mipmap_filter: wgpu::FilterMode::Linear,
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        self.background_color.as_entire_buffer_binding(),
                    ),
                },
            ],
        });
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        marching_squares_processor: &MarchingSquaresProcessor<Quads>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Marching Squares Liquid Quad Renderer Render Pass"),
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
        render_pass.set_bind_group(0, &self.quads_bind_group, &[]);
        render_pass.set_bind_group(1, &self.meta_field_bind_group, &[]);
        render_pass.set_bind_group(2, &self.background_bind_group, &[]);
        render_pass.draw_indirect(marching_squares_processor.indirect_buffer.buffer(), 0);
    }
}
