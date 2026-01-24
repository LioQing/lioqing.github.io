use std::sync::mpsc;

use ahash::HashMap;
use glam::*;
use strum::IntoDiscriminant;
use wasm_bindgen::{JsCast as _, UnwrapThrowExt as _};

use crate::{
    controller::{
        BackgroundController, ExperienceController, PanelController, ProjectController,
        SkillsController,
    },
    delta_time::DeltaTime,
    ext::{
        CanvasExt as _, HtmlCollectionExt as _, SurfaceConfigurationExt as _, Vec4Ext, WindowExt,
    },
    frame::{self, FrameMetadata},
    gpu::Gpu,
    grid::{
        GridMetadata, GridState,
        pipeline::{GridProcessor, GridRenderer},
    },
    mar_sq::{
        line_segment::LineSegments,
        pipeline::{
            MarchingSquaresLiquidQuadRenderer, MarchingSquaresProcessor,
            MarchingSquaresShapeRenderer,
        },
        quad::Quads,
    },
    meta_field::MetaField,
    meta_shape::{MetaBall, MetaShapes},
    mouse::Mouse,
    pipeline::{
        BackgroundImageRenderer, BackgroundSvgRenderer, GaussianBlurPipeline, MetaFieldGrad,
        MetaFieldMag, MetaFieldProcessor, MetaFieldRenderer,
    },
    texture_blitter::TextureBlitter,
    theme::{Theme, ThemePropertyName},
};

#[derive(Debug, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[strum_discriminants(name(BackgroundEventType))]
pub enum BackgroundEvent {
    Resize,
    MouseMove(IVec2),
}

pub struct Background {
    gpu: Gpu,
    background_events: mpsc::Receiver<BackgroundEvent>,
    frame_timer: web_time::Instant,
    fps_display_counter: u32,
    mouse: Mouse,

    // Controller
    experience_controller: ExperienceController,
    skills_controller: SkillsController,
    background_controller: BackgroundController,
    panel_controller: PanelController,

    // Pipelines
    skills_renderer: BackgroundSvgRenderer,
    zero_one_background_renderer: BackgroundImageRenderer,
    skills_background_renderer: BackgroundImageRenderer,
    blur: GaussianBlurPipeline,
    // grid_processor: GridProcessor,
    // grid_renderer: GridRenderer,
    meta_field_processor: MetaFieldProcessor,
    meta_field_renderer: MetaFieldRenderer<MetaFieldGrad>,
    marching_squares_processor: MarchingSquaresProcessor<Quads>,
    marching_squares_shape_renderer: MarchingSquaresShapeRenderer<Quads>,
    marching_squares_liquid_quad_renderer: MarchingSquaresLiquidQuadRenderer,
    surface_blitter: TextureBlitter,

    // Data
    frame_metadata: FrameMetadata,
    background: wgpu::Texture,
    delta_time: DeltaTime,
    grid_state: GridState,
    grid_metadata: GridMetadata,
    meta_shapes: MetaShapes,
    meta_field: MetaField,
    // line_segments: LineSegments,
    quads: Quads,
}

impl Background {
    pub async fn new(
        gpu: Gpu,
        canvas: web_sys::HtmlCanvasElement,
        background_events: mpsc::Receiver<BackgroundEvent>,
    ) -> Self {
        let experience_controller = ExperienceController::new().await;
        let skills_controller = SkillsController::new();
        let _project_controller = ProjectController::new().await;

        let background_controller = BackgroundController::new();
        let mut panel_controller = PanelController::new();

        let frame_metadata = FrameMetadata::new(&gpu.device, canvas.size(), IVec2::ZERO);

        let background = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Background Texture"),
            size: wgpu::Extent3d {
                width: frame_metadata.resolution().x,
                height: frame_metadata.resolution().y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: gpu.config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let delta_time = DeltaTime::new(&gpu.device);
        let grid_metadata = GridMetadata::new(&gpu.device, &frame_metadata);
        let grid_state = GridState::new(&gpu.device, &grid_metadata);

        let frame_timer = web_time::Instant::now();

        let mouse = Mouse::new(frame_metadata.resolution().as_vec2() / 2.0);

        let meta_shapes = MetaShapes::new_with_controller(&gpu.device, &mut panel_controller);

        const CELL_SIZE: u32 = 4;
        let meta_field = MetaField::new(&gpu.device, &frame_metadata, CELL_SIZE);

        // let line_segments = LineSegments::new(&gpu.device, &meta_field);
        let quads = Quads::new(&gpu.device, &meta_field);

        let skills_renderer =
            BackgroundSvgRenderer::new_skills(&gpu.device, gpu.config.format).await;

        let zero_one_background_renderer =
            BackgroundImageRenderer::new_zero_one(&gpu.device, &gpu.queue, gpu.config.format).await;

        let skills_background_renderer =
            BackgroundImageRenderer::new_skills(&gpu.device, &gpu.queue, gpu.config.format).await;

        let blur = GaussianBlurPipeline::new(&gpu.device, &frame_metadata, gpu.config.format);

        // let grid_processor = GridProcessor::new(
        //     &gpu.device,
        //     &frame_metadata,
        //     &grid_metadata,
        //     &delta_time,
        //     &grid_state,
        //     mouse.position(),
        // );

        // let grid_renderer = GridRenderer::new(
        //     &gpu.device,
        //     &frame_metadata,
        //     &grid_metadata,
        //     &grid_state,
        //     gpu.config.format,
        // );

        let meta_field_processor =
            MetaFieldProcessor::new(&gpu.device, &frame_metadata, &meta_shapes, &meta_field);

        let meta_field_renderer =
            MetaFieldRenderer::new(&gpu.device, &meta_field, gpu.config.format);

        let marching_squares_processor =
            MarchingSquaresProcessor::new(&gpu.device, &meta_field, &quads);

        let marching_squares_shape_renderer = MarchingSquaresShapeRenderer::new(
            &gpu.device,
            &frame_metadata,
            &quads,
            gpu.config.format,
        );

        let marching_squares_liquid_quad_renderer = MarchingSquaresLiquidQuadRenderer::new(
            &gpu.device,
            &frame_metadata,
            &quads,
            &meta_field,
            blur.output_view(),
            Theme::current()
                .properties()
                .get(&ThemePropertyName::Background)
                .expect_throw("background color")
                .vec4()
                .expect("background color vector")
                .xyz(),
            gpu.config.format,
        );

        let surface_blitter = TextureBlitter::new(&gpu.device, gpu.config.format);

        Self {
            gpu,
            background_events,
            frame_timer,
            fps_display_counter: 0,
            mouse,

            skills_renderer,
            zero_one_background_renderer,
            skills_background_renderer,
            blur,
            // grid_processor,
            // grid_renderer,
            meta_field_processor,
            meta_field_renderer,
            marching_squares_processor,
            marching_squares_shape_renderer,
            marching_squares_liquid_quad_renderer,
            surface_blitter,

            frame_metadata,
            background,
            delta_time,
            grid_state,
            grid_metadata,
            meta_shapes,
            meta_field,
            // line_segments,
            quads,

            panel_controller,
            experience_controller,
            background_controller,
            skills_controller,
        }
    }

    pub fn update(&mut self) {
        let delta_time = self.frame_timer.elapsed().as_secs_f32().min(33e-3);
        self.frame_timer = web_time::Instant::now();

        self.handle_event();
        self.handle_update(delta_time);
        if self.should_render() {
            self.handle_render();
        }

        // FPS display
        self.fps_display_counter += 1;
        if self.fps_display_counter >= 60 {
            let fps = 1.0 / delta_time;
            if cfg!(debug_assertions) {
                log::info!("Background FPS: {:.2}", fps);
            }
            self.fps_display_counter = 0;
        }
    }

    fn handle_event(&mut self) {
        let event_map = self
            .background_events
            .try_iter()
            .map(|event| (event.discriminant(), event))
            .collect::<HashMap<_, _>>();

        for event in event_map.into_values() {
            match event {
                BackgroundEvent::Resize => self.handle_resize(),
                BackgroundEvent::MouseMove(pos) => self.handle_mouse_move(pos),
            }
        }
    }

    fn handle_resize(&mut self) {
        let window = web_sys::window().expect_throw("window");
        let size = window.size().as_uvec2();
        self.gpu.config.width = size.x;
        self.gpu.config.height = size.y;

        if !self.gpu.config.is_valid() {
            return;
        }

        self.gpu
            .surface
            .configure(&self.gpu.device, &self.gpu.config);
        log::debug!("Resized to {size}");

        self.frame_metadata
            .update(&self.gpu.queue, self.gpu.config.size(), window.scroll_pos());

        self.background = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Background Texture"),
            size: wgpu::Extent3d {
                width: self.frame_metadata.resolution().x,
                height: self.frame_metadata.resolution().y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.gpu.config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        self.experience_controller.resize();

        self.grid_metadata
            .update(&self.gpu.queue, &self.frame_metadata);

        self.grid_state
            .resize(&self.gpu.device, &self.grid_metadata);

        self.panel_controller
            .resize(&mut self.meta_shapes, window.scroll_pos());

        self.meta_field
            .resize(&self.gpu.device, &self.frame_metadata);

        self.blur.resize(&self.gpu.device, &self.frame_metadata);

        // self.grid_processor.recreate_bind_group(
        //     &self.gpu.device,
        //     &self.frame_metadata,
        //     &self.grid_metadata,
        //     &self.delta_time,
        //     &self.grid_state,
        // );

        // self.grid_renderer.recreate_bind_group(
        //     &self.gpu.device,
        //     &self.frame_metadata,
        //     &self.grid_metadata,
        //     &self.grid_state,
        // );

        self.meta_field_processor.recreate_bind_group(
            &self.gpu.device,
            &self.frame_metadata,
            &self.meta_shapes,
            &self.meta_field,
        );

        self.meta_field_renderer
            .recreate_bind_group(&self.gpu.device, &self.meta_field);

        self.marching_squares_processor.recreate_bind_group(
            &self.gpu.device,
            &self.meta_field,
            &self.quads,
        );

        self.marching_squares_shape_renderer.recreate_bind_group(
            &self.gpu.device,
            &self.frame_metadata,
            &self.quads,
        );

        self.marching_squares_liquid_quad_renderer
            .recreate_bind_group(
                &self.gpu.device,
                &self.frame_metadata,
                &self.quads,
                &self.meta_field,
                self.blur.output_view(),
            );
    }

    fn handle_mouse_move(&mut self, pos: IVec2) {
        self.mouse.set_target(pos.as_vec2());
    }

    fn handle_update(&mut self, delta_time: f32) {
        self.frame_metadata.update(
            &self.gpu.queue,
            self.gpu.config.size(),
            web_sys::window().expect_throw("window").scroll_pos(),
        );

        self.delta_time.update(&self.gpu.queue, delta_time);

        self.mouse.update(&self.frame_metadata, delta_time);

        // self.grid_processor.update_target(
        //     &self.gpu.queue,
        //     &self.frame_metadata,
        //     &self.mouse,
        //     delta_time,
        // );

        self.panel_controller
            .update(&mut self.meta_shapes, delta_time);

        self.meta_shapes.balls_mut()[0] = MetaBall {
            center: self.mouse.position(),
            radius: 18.0,
            hidden: if self.mouse.hidden() { 1 } else { 0 },
        };

        // TODO: Update only if needed
        self.meta_shapes.ensure_buffer(&self.gpu.queue);

        self.background_controller
            .update(&self.frame_metadata, &self.zero_one_background_renderer);
        self.experience_controller.update();
        self.skills_controller.update();
    }

    fn should_render(&self) -> bool {
        self.gpu.config.is_valid()
    }

    fn handle_render(&mut self) {
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Background Command Encoder"),
            });

        // Render to background
        {
            let view = self
                .background
                .create_view(&wgpu::TextureViewDescriptor::default());

            {
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                Theme::current()
                                    .properties()
                                    .get(&ThemePropertyName::Background)
                                    .expect_throw("background color")
                                    .vec4()
                                    .expect_throw("background color vector")
                                    .to_wgpu_color(),
                            ),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
            }

            self.zero_one_background_renderer.render(
                &self.gpu.device,
                &self.gpu.queue,
                &mut encoder,
                &view,
                &self.frame_metadata,
                self.background_controller.zero_one_position().as_ivec2(),
            );

            self.skills_background_renderer.render(
                &self.gpu.device,
                &self.gpu.queue,
                &mut encoder,
                &view,
                &self.frame_metadata,
                self.background_controller.skills_position().as_ivec2(),
            );

            self.skills_renderer.render(
                &self.gpu.device,
                &self.gpu.queue,
                &mut encoder,
                &view,
                &self.frame_metadata,
                self.skills_controller.top_left().as_ivec2(),
                self.skills_controller.bottom_right().as_ivec2(),
            );

            // self.grid_processor
            //     .process(&mut encoder, self.grid_metadata.resolution());

            // self.grid_renderer
            //     .render(&mut encoder, &view, self.grid_metadata.resolution());

            self.meta_field_processor
                .process(&mut encoder, self.meta_field.resolution());

            // self.meta_field_renderer.render(&mut encoder, &view);

            self.marching_squares_processor.process(
                &self.gpu.queue,
                &mut encoder,
                self.meta_field.resolution(),
            );

            self.blur.blur(
                &self.gpu.device,
                &self.gpu.queue,
                &mut encoder,
                &self
                    .background
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );

            // self.marching_squares_shape_renderer.render(
            //     &mut encoder,
            //     &view,
            //     &self.marching_squares_processor,
            // );
        }

        // Render to screen
        {
            let texture = match self.gpu.surface.get_current_texture() {
                Ok(texture) => texture,
                Err(e) => {
                    log::error!("Failed to get current texture: {e:?}");
                    return;
                }
            };
            let view = texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.surface_blitter.copy_full(
                &self.gpu.device,
                &self.gpu.queue,
                &mut encoder,
                &self
                    .background
                    .create_view(&wgpu::TextureViewDescriptor::default()),
                &view,
                &self.frame_metadata,
            );

            self.marching_squares_liquid_quad_renderer.render(
                &mut encoder,
                &view,
                &self.marching_squares_processor,
            );

            self.gpu.queue.submit(Some(encoder.finish()));
            texture.present();
        }

        if let Err(e) = self.gpu.device.poll(wgpu::PollType::Poll) {
            log::error!("Failed to submit commands to GPU: {e}");
        }
    }
}
