use glam::*;
use vello_svg::vello;
use wasm_bindgen::UnwrapThrowExt;
use wgpu::util::DeviceExt as _;

use crate::mouse::Mouse;
use crate::texture_blitter::TextureBlitter;
use crate::{frame::FrameMetadata, meta_field::MetaField, meta_shape::MetaShapes};

pub const RADIUS: f64 = 36.0;
pub const FADE_DIST: f64 = 24.0;
pub const HEIGHT: f64 = 36.0;

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
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
                    resource: meta_shapes.balls_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: meta_shapes.boxes_buffer().as_entire_binding(),
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
                    ("base_radius", RADIUS),
                    ("fade_dist", FADE_DIST),
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
                    resource: meta_shapes.balls_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: meta_shapes.boxes_buffer().as_entire_binding(),
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

        let bind_group_layout =
            Self::create_bind_group_layout(device, Some("Meta Field Renderer Bind Group Layout"));

        let bind_group = Self::create_bind_group(
            device,
            Some("Meta Field Renderer Bind Group"),
            &bind_group_layout,
            meta_field,
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Meta Field Renderer Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let compilation_options = wgpu::PipelineCompilationOptions {
            constants: &[
                ("base_radius", RADIUS),
                ("fade_dist", FADE_DIST),
                ("base_height", HEIGHT),
            ],
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
        self.bind_group = Self::create_bind_group(
            device,
            Some("Meta Field Renderer Bind Group"),
            &self.bind_group_layout,
            meta_field,
        );
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

    pub fn create_bind_group_layout(
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
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
        })
    }

    pub fn create_bind_group(
        device: &wgpu::Device,
        label: Option<&str>,
        layout: &wgpu::BindGroupLayout,
        meta_field: &MetaField,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout,
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
        })
    }
}

pub struct BackgroundSvgRenderer {
    scene: vello::Scene,
    renderer: vello::Renderer,
    background_blitter: TextureBlitter,
    intermediate_texture: wgpu::Texture,
}

impl BackgroundSvgRenderer {
    pub fn new(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let scene =
            vello_svg::render(include_str!("../assets/test.svg")).expect_throw("background svg");

        let renderer = vello::Renderer::new(
            device,
            vello::RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .expect_throw("vello renderer");

        let intermediate_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Background SVG Intermediate Texture"),
            size: wgpu::Extent3d {
                width: frame_metadata.resolution().x,
                height: frame_metadata.resolution().y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let background_blitter = TextureBlitter::new(device, texture_format);

        Self {
            scene,
            renderer,
            background_blitter,
            intermediate_texture,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, frame_metadata: &FrameMetadata) {
        self.intermediate_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Background SVG Intermediate Texture"),
            size: wgpu::Extent3d {
                width: frame_metadata.resolution().x,
                height: frame_metadata.resolution().y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        background_view: &wgpu::TextureView,
        frame_metadata: &FrameMetadata,
        mosue: &Mouse,
    ) {
        if let Err(e) = self.renderer.render_to_texture(
            device,
            queue,
            &self.scene,
            &self
                .intermediate_texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            &vello::RenderParams {
                base_color: vello::peniko::color::palette::css::TRANSPARENT,
                width: frame_metadata.resolution().x,
                height: frame_metadata.resolution().y,
                antialiasing_method: vello::AaConfig::Area,
            },
        ) {
            log::error!("Failed to render background SVG: {e}");
        }

        // TODO: This is a temporary position for this test svg
        let init_pos = IVec2::new(0, frame_metadata.resolution().y as i32);
        let parallax_offset = -(frame_metadata.top_left().as_vec2() * 0.5).as_ivec2();
        let mouse_offset = ((mosue.position() - frame_metadata.center()) * 0.01).as_ivec2();

        self.background_blitter.copy(
            device,
            queue,
            encoder,
            &self
                .intermediate_texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            background_view,
            frame_metadata,
            init_pos + parallax_offset + mouse_offset,
            Vec2::ONE,
        );
    }
}

#[derive(Debug)]
pub struct BackgroundImageRenderer {
    background_blitter: TextureBlitter,
    image: wgpu::Texture,
    size: UVec2,
}

impl BackgroundImageRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
        data: &[u8],
    ) -> Self {
        let bytes = image::load_from_memory(data)
            .expect("load background image")
            .to_rgba8();

        let (width, height) = bytes.dimensions();
        let size = UVec2::new(width, height);

        let image = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Background Image Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            &bytes,
        );

        let background_blitter = TextureBlitter::new(device, texture_format);

        Self {
            background_blitter,
            image,
            size,
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        background_view: &wgpu::TextureView,
        frame_metadata: &FrameMetadata,
        offset: i32,
    ) {
        let (scale, center) = self.get_scale_and_center(frame_metadata);
        let top_left =
            (center - frame_metadata.top_left().as_vec2() * 0.5).as_ivec2() + IVec2::new(0, offset);

        self.background_blitter.copy(
            device,
            queue,
            encoder,
            &self
                .image
                .create_view(&wgpu::TextureViewDescriptor::default()),
            background_view,
            frame_metadata,
            top_left,
            scale,
        );
    }

    pub fn get_scale_and_center(&self, frame_metadata: &FrameMetadata) -> (Vec2, Vec2) {
        let scale =
            Vec2::splat((frame_metadata.resolution().x as f32 / self.size.x as f32).max(0.5));
        let center = (frame_metadata.resolution().as_vec2() - self.size.as_vec2() * scale) * 0.5;
        (scale, center)
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }
}

#[derive(Debug)]
pub struct GaussianBlurPipeline {
    render_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_buffer_x: wgpu::Buffer,
    params_buffer_y: wgpu::Buffer,
    ping_texture: wgpu::Texture,
    ping_view: wgpu::TextureView,
    pong_texture: wgpu::Texture,
    pong_view: wgpu::TextureView,
    texture_format: wgpu::TextureFormat,
}

impl GaussianBlurPipeline {
    pub fn new(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gaussian Blur Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/gaussian_blur.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Gaussian Blur Bind Group Layout"),
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

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gaussian Blur Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gaussian Blur Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vert_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("frag_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Gaussian Blur Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let params_buffer_x = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gaussian Blur Params Buffer X"),
            contents: bytemuck::bytes_of(&Vec4::new(1.0, 0.0, 0.0, 0.0)),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let params_buffer_y = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gaussian Blur Params Buffer Y"),
            contents: bytemuck::bytes_of(&Vec4::new(0.0, 1.0, 0.0, 0.0)),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let (ping_texture, ping_view) = Self::create_target_texture(
            device,
            frame_metadata,
            texture_format,
            "Gaussian Blur Ping Texture",
        );

        let (pong_texture, pong_view) = Self::create_target_texture(
            device,
            frame_metadata,
            texture_format,
            "Gaussian Blur Pong Texture",
        );

        Self {
            render_pipeline,
            bind_group_layout,
            sampler,
            params_buffer_x,
            params_buffer_y,
            ping_texture,
            ping_view,
            pong_texture,
            pong_view,
            texture_format,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, frame_metadata: &FrameMetadata) {
        let (ping_texture, ping_view) = Self::create_target_texture(
            device,
            frame_metadata,
            self.texture_format,
            "Gaussian Blur Ping Texture",
        );
        let (pong_texture, pong_view) = Self::create_target_texture(
            device,
            frame_metadata,
            self.texture_format,
            "Gaussian Blur Pong Texture",
        );

        self.ping_texture = ping_texture;
        self.ping_view = ping_view;
        self.pong_texture = pong_texture;
        self.pong_view = pong_view;
    }

    pub fn blur(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
    ) {
        self.run_pass(
            device,
            encoder,
            input_view,
            &self.ping_view,
            Vec2::new(1.0, 0.0),
        );
        self.run_pass(
            device,
            encoder,
            &self.ping_view,
            &self.pong_view,
            Vec2::new(0.0, 1.0),
        );
    }

    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.pong_view
    }

    fn run_pass(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        src_view: &wgpu::TextureView,
        dst_view: &wgpu::TextureView,
        direction: Vec2,
    ) {
        let params_buffer = if direction.x.abs() > direction.y.abs() {
            &self.params_buffer_x
        } else {
            &self.params_buffer_y
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gaussian Blur Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Gaussian Blur Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst_view,
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
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn create_target_texture(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        texture_format: wgpu::TextureFormat,
        label: &str,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: frame_metadata.resolution().x,
                height: frame_metadata.resolution().y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}
