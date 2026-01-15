use glam::*;

use crate::{frame::FrameMetadata, meta_field::MetaField, meta_shape::MetaShapes};

#[derive(Debug)]
pub struct MetaFieldProcessor {
    workgroup_size: UVec2,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl MetaFieldProcessor {
    pub fn new(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        meta_shapes: &MetaShapes,
        meta_field: &MetaField,
    ) -> Self {
        let workgroup_size = {
            let mut max_size = None;

            for i in (2..=8).rev() {
                let size = 1 << i;
                if device.limits().max_compute_workgroup_size_x >= size
                    && device.limits().max_compute_workgroup_size_y >= size
                    && device.limits().max_compute_invocations_per_workgroup >= size * size
                {
                    max_size = Some(UVec2::splat(size));
                    break;
                }
            }

            log::debug!("Chosen workgroup size for MetaFieldProcessor: {max_size:?}");

            max_size.expect("device must support workgroup size of at least 4x4")
        };

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Meta Field Processor Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/meta_field_process.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Meta Field Processor Bind Group Layout"),
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
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: meta_field.texture().format(),
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
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
            label: Some("Meta Field Processor Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: meta_field.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &meta_field
                            .texture()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: meta_shapes.buffer().as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Meta Field Processor Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Meta Field Processor Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &[
                    ("workgroup_size_x", workgroup_size.x as f64),
                    ("workgroup_size_y", workgroup_size.y as f64),
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
        }
    }

    pub fn recreate_bind_group(
        &mut self,
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        meta_shapes: &MetaShapes,
        meta_field: &MetaField,
    ) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Meta Field Processor Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_metadata.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: meta_field.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &meta_field
                            .texture()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: meta_shapes.buffer().as_entire_binding(),
                },
            ],
        });
    }

    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, resolution: UVec2) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Meta Field Processor Compute Pass"),
            ..Default::default()
        });

        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);

        compute_pass.dispatch_workgroups(
            resolution.x.div_ceil(self.workgroup_size.x),
            resolution.y.div_ceil(self.workgroup_size.y),
            1,
        );
    }
}

pub trait MetaFieldRenderType {
    const SHADER: &'static str;
}

#[derive(Debug)]
pub struct MetaFieldMag;

#[derive(Debug)]
pub struct MetaFieldGrad;

impl MetaFieldRenderType for MetaFieldMag {
    const SHADER: &'static str = include_str!("shader/meta_field.wgsl");
}

impl MetaFieldRenderType for MetaFieldGrad {
    const SHADER: &'static str = include_str!("shader/meta_field_grad.wgsl");
}

#[derive(Debug)]
pub struct MetaFieldRenderer<T: MetaFieldRenderType> {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    phantom: std::marker::PhantomData<T>,
}

impl<T: MetaFieldRenderType> MetaFieldRenderer<T> {
    pub fn new(
        device: &wgpu::Device,
        meta_field: &MetaField,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let render_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Meta Field Renderer Shader Module"),
            source: wgpu::ShaderSource::Wgsl(T::SHADER.into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Meta Field Renderer Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Meta Field Renderer Bind Group"),
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
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Meta Field Renderer Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compilation_options = wgpu::PipelineCompilationOptions {
            constants: &[("cell_size", meta_field.cell_size() as f64)],
            ..Default::default()
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Meta Field Renderer Render Pipeline"),
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
            bind_group_layout,
            bind_group,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn recreate_bind_group(&mut self, device: &wgpu::Device, meta_field: &MetaField) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Meta Field Renderer Bind Group"),
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
            ],
        });
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Meta Field Renderer Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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
        render_pass.draw(0..3, 0..1);
    }
}
