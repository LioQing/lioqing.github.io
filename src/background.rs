use std::sync::mpsc;

use ahash::HashMap;
use glam::*;
use strum::IntoDiscriminant;
use wasm_bindgen::UnwrapThrowExt as _;

use crate::{
    ext::{CanvasExt as _, SurfaceConfigurationExt as _, Vec4Ext, WindowExt},
    frame::FrameMetadata,
    gpu::Gpu,
    line_segment::LineSegments,
    meta_field::MetaField,
    meta_shape::{MetaBall, MetaBox, MetaLine, MetaShapes},
    mouse::Mouse,
    pipeline::{
        LineSegmentRenderer, MarchingSquaresProcessor, MetaFieldProcessor, MetaFieldRenderer,
    },
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
    meta_field_processor: MetaFieldProcessor,
    meta_field_renderer: MetaFieldRenderer,
    marching_squares_processor: MarchingSquaresProcessor,
    line_segment_renderer: LineSegmentRenderer,
    frame_metadata: FrameMetadata,
    meta_shapes: MetaShapes,
    meta_field: MetaField,
    line_segments: LineSegments,
    frame_timer: web_time::Instant,
    mouse: Mouse,
}

impl Background {
    pub fn new(
        gpu: Gpu,
        canvas: web_sys::HtmlCanvasElement,
        background_events: mpsc::Receiver<BackgroundEvent>,
    ) -> Self {
        let frame_metadata = FrameMetadata::new(&gpu.device, canvas.size(), IVec2::ZERO);

        let mut meta_shapes = MetaShapes::new(&gpu.device, 1, 0, 0);
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

        let meta_field = MetaField::new(&gpu.device, &frame_metadata, 8);

        let line_segments = LineSegments::new(&gpu.device, &meta_field);

        let meta_field_processor =
            MetaFieldProcessor::new(&gpu.device, &frame_metadata, &meta_shapes, &meta_field);

        let meta_field_renderer =
            MetaFieldRenderer::new(&gpu.device, &meta_field, gpu.config.format);

        let marching_squares_processor =
            MarchingSquaresProcessor::new(&gpu.device, &meta_field, &line_segments);

        let line_segment_renderer = LineSegmentRenderer::new(
            &gpu.device,
            &frame_metadata,
            &line_segments,
            gpu.config.format,
        );

        let frame_timer = web_time::Instant::now();

        let mouse = Mouse::new(frame_metadata.resolution().as_vec2() / 2.0);

        Self {
            gpu,
            background_events,
            meta_field_processor,
            meta_field_renderer,
            marching_squares_processor,
            line_segment_renderer,
            frame_metadata,
            meta_shapes,
            meta_field,
            line_segments,
            frame_timer,
            mouse,
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
            &self.line_segments,
        );

        self.line_segment_renderer.recreate_bind_group(
            &self.gpu.device,
            &self.frame_metadata,
            &self.line_segments,
        );
    }

    fn handle_mouse_move(&mut self, pos: IVec2) {
        self.mouse.set_target(pos.as_vec2());
    }

    fn handle_update(&mut self, delta_time: f32) {
        let window = web_sys::window().expect_throw("window");

        self.frame_metadata
            .update(&self.gpu.queue, self.gpu.config.size(), window.scroll_pos());

        self.mouse.update(&self.frame_metadata, delta_time);

        self.meta_shapes.balls_mut()[0] = MetaBall {
            position: self.mouse.position(),
            radius: 36.0,
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
                label: Some("Line Segment Renderer Render Pass"),
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

        self.meta_field_processor
            .process(&mut encoder, self.meta_field.resolution());

        // self.meta_field_renderer.render(&mut encoder, &view);

        self.marching_squares_processor.process(
            &self.gpu.queue,
            &mut encoder,
            self.meta_field.resolution(),
        );

        self.line_segment_renderer
            .render(&mut encoder, &view, &self.marching_squares_processor);

        self.gpu.queue.submit(Some(encoder.finish()));
        texture.present();

        if let Err(e) = self.gpu.device.poll(wgpu::PollType::Poll) {
            log::error!("Failed to submit commands to GPU: {e}");
        }
    }
}
