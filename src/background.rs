use std::sync::mpsc;

use ahash::HashMap;
use glam::*;
use strum::IntoDiscriminant;
use wasm_bindgen::{JsCast as _, UnwrapThrowExt as _};

use crate::{
    delta_time::{self, DeltaTime},
    ext::{
        CanvasExt as _, HtmlCollectionExt as _, SurfaceConfigurationExt as _, Vec4Ext, WindowExt,
    },
    frame::FrameMetadata,
    gpu::Gpu,
    grid::{
        GridMetadata, GridState,
        pipeline::{GridProcessor, GridRenderer},
    },
    mar_sq::{
        line_segment::LineSegments,
        pipeline::{MarchingSquaresProcessor, MarchingSquaresShapeRenderer},
        quad::Quads,
    },
    meta_field::MetaField,
    meta_shape::{MetaBall, MetaBox, MetaLine, MetaShapes},
    mouse::Mouse,
    pipeline::{MetaFieldGrad, MetaFieldMag, MetaFieldProcessor, MetaFieldRenderer},
    theme::{Theme, ThemePropertyName},
};

#[derive(Debug, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
#[strum_discriminants(name(BackgroundEventType))]
pub enum BackgroundEvent {
    Resize,
    MouseMove(IVec2),
}

#[derive(Debug)]
pub struct Background {
    gpu: Gpu,
    background_events: mpsc::Receiver<BackgroundEvent>,
    panels: Vec<web_sys::HtmlElement>,
    frame_timer: web_time::Instant,
    mouse: Mouse,

    // Pipelines
    grid_processor: GridProcessor,
    grid_renderer: GridRenderer,
    meta_field_processor: MetaFieldProcessor,
    meta_field_renderer: MetaFieldRenderer<MetaFieldGrad>,
    marching_squares_processor: MarchingSquaresProcessor<Quads>,
    marching_squares_shape_renderer: MarchingSquaresShapeRenderer<Quads>,

    // Data
    frame_metadata: FrameMetadata,
    delta_time: DeltaTime,
    grid_state: GridState,
    grid_metadata: GridMetadata,
    meta_shapes: MetaShapes,
    meta_field: MetaField,
    line_segments: LineSegments,
    quads: Quads,
}

impl Background {
    pub fn new(
        gpu: Gpu,
        canvas: web_sys::HtmlCanvasElement,
        background_events: mpsc::Receiver<BackgroundEvent>,
    ) -> Self {
        let frame_metadata = FrameMetadata::new(&gpu.device, canvas.size(), IVec2::ZERO);
        let delta_time = DeltaTime::new(&gpu.device);
        let grid_metadata = GridMetadata::new(&gpu.device, &frame_metadata);
        let grid_state = GridState::new(&gpu.device, &grid_metadata);

        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let panels = document
            .get_elements_by_class_name("panel")
            .iter()
            .map(|el| el.dyn_into::<web_sys::HtmlElement>().unwrap_throw())
            .collect::<Vec<_>>();

        let frame_timer = web_time::Instant::now();

        let mouse = Mouse::new(frame_metadata.resolution().as_vec2() / 2.0);

        let mut meta_shapes = MetaShapes::new(&gpu.device, 1, 0, panels.len());

        meta_shapes.update_from_panels(window.scroll_pos(), &panels);
        // let mut meta_shapes = MetaShapes::new(&gpu.device, 2, 1, 1);
        // meta_shapes.balls_mut()[1] = MetaBall {
        //     position: vec2(200.0, 200.0),
        //     radius: 75.0,
        // };
        // meta_shapes.lines_mut()[0] = MetaLine {
        //     start: vec2(600.0, 100.0),
        //     end: vec2(700.0, 400.0),
        //     radius: 36.0,
        // };
        // meta_shapes.boxes_mut()[0] = MetaBox {
        //     min: vec2(0.0, 1300.0),
        //     max: vec2(800.0, 2100.0),
        //     radius: 48.0,
        // };

        const CELL_SIZE: u32 = 4;
        const FADE_DIST: u32 = 36;
        let meta_field = MetaField::new(&gpu.device, &frame_metadata, CELL_SIZE, FADE_DIST);

        let line_segments = LineSegments::new(&gpu.device, &meta_field);
        let quads = Quads::new(&gpu.device, &meta_field);

        let grid_processor = GridProcessor::new(
            &gpu.device,
            &frame_metadata,
            &grid_metadata,
            &delta_time,
            &grid_state,
            mouse.position(),
        );

        let grid_renderer = GridRenderer::new(
            &gpu.device,
            &frame_metadata,
            &grid_metadata,
            &grid_state,
            gpu.config.format,
        );

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

        Self {
            gpu,
            background_events,
            panels,
            frame_timer,
            mouse,

            grid_processor,
            grid_renderer,
            meta_field_processor,
            meta_field_renderer,
            marching_squares_processor,
            marching_squares_shape_renderer,

            frame_metadata,
            delta_time,
            grid_state,
            grid_metadata,
            meta_shapes,
            meta_field,
            line_segments,
            quads,
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

        self.grid_metadata
            .update(&self.gpu.queue, &self.frame_metadata);

        self.grid_state
            .resize(&self.gpu.device, &self.grid_metadata);

        self.grid_processor.recreate_bind_group(
            &self.gpu.device,
            &self.frame_metadata,
            &self.grid_metadata,
            &self.delta_time,
            &self.grid_state,
        );

        self.grid_renderer.recreate_bind_group(
            &self.gpu.device,
            &self.frame_metadata,
            &self.grid_metadata,
            &self.grid_state,
        );

        self.meta_field
            .resize(&self.gpu.device, &self.frame_metadata);

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

        self.meta_shapes
            .update_from_panels(window.scroll_pos(), &self.panels);
    }

    fn handle_mouse_move(&mut self, pos: IVec2) {
        self.mouse.set_target(pos.as_vec2());
    }

    fn handle_update(&mut self, delta_time: f32) {
        let window = web_sys::window().expect_throw("window");

        self.frame_metadata
            .update(&self.gpu.queue, self.gpu.config.size(), window.scroll_pos());

        self.delta_time.update(&self.gpu.queue, delta_time);

        self.mouse.update(&self.frame_metadata, delta_time);

        self.grid_processor.update_target(
            &self.gpu.queue,
            &self.frame_metadata,
            self.mouse.position(),
            delta_time,
        );

        self.meta_shapes.balls_mut()[0] = MetaBall {
            position: self.mouse.position(),
            radius: 48.0,
        };

        self.meta_shapes.ensure_buffer(&self.gpu.queue);
    }

    fn should_render(&self) -> bool {
        self.gpu.config.is_valid()
    }

    fn handle_render(&mut self) {
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

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Background Command Encoder"),
            });

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

        self.grid_processor
            .process(&mut encoder, self.grid_metadata.resolution());

        self.grid_renderer
            .render(&mut encoder, &view, self.grid_metadata.resolution());

        self.meta_field_processor
            .process(&mut encoder, self.meta_field.resolution());

        self.meta_field_renderer.render(&mut encoder, &view);

        self.marching_squares_processor.process(
            &self.gpu.queue,
            &mut encoder,
            self.meta_field.resolution(),
        );

        // self.marching_squares_shape_renderer.render(
        //     &mut encoder,
        //     &view,
        //     &self.marching_squares_processor,
        // );

        self.gpu.queue.submit(Some(encoder.finish()));
        texture.present();

        if let Err(e) = self.gpu.device.poll(wgpu::PollType::Poll) {
            log::error!("Failed to submit commands to GPU: {e}");
        }
    }
}
