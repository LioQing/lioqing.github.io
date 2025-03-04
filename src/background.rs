use std::{
    num::NonZeroU32,
    sync::{mpsc::Receiver, Arc},
};

use image::GenericImageView;
use itertools::Itertools;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::Window,
};

use crate::state::Theme;

pub enum BackgroundCommand {
    CreateCanvas(&'static str),
    Destroy,
    Resize(f32, f32),
    SetTheme(Theme),
    PointerMove(f32, f32),
    PointerDown(f32),
}

pub fn background(rx: Receiver<BackgroundCommand>) {
    let event_loop = EventLoop::new().expect("create event loop");

    log::debug!("Starting background");
    #[cfg(not(target_arch = "wasm32"))]
    {
        event_loop
            .run_app(&mut BackgroundApp::Uninit { rx })
            .expect("run app");
    }
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn_app(BackgroundApp::Uninit { rx });
    }
}

#[derive(Debug, Default)]
enum BackgroundApp {
    #[default]
    Invalid,
    Uninit {
        rx: Receiver<BackgroundCommand>,
    },
    Init {
        rx: Receiver<BackgroundCommand>,
        window: Arc<Window>,
        runner: Receiver<Runner>,
    },
    Running {
        rx: Receiver<BackgroundCommand>,
        window: Arc<Window>,
        canvas_id: Option<&'static str>,
        runner: Runner,
        input: Input,
        frame_timer: chrono::DateTime<chrono::Local>,
        #[cfg(target_arch = "wasm32")]
        redraw_closure: wasm_bindgen::prelude::Closure<dyn FnMut()>,
    },
}

impl ApplicationHandler for BackgroundApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if !matches!(self, BackgroundApp::Uninit { .. }) {
            return;
        }

        let BackgroundApp::Uninit { rx } = std::mem::take(self) else {
            unreachable!()
        };

        // Create window.
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .expect("create window"),
        );

        // Create runner.
        let runner = {
            let (tx, rx) = std::sync::mpsc::channel();
            let window = window.clone();
            #[cfg(target_arch = "wasm32")]
            wasm_bindgen_futures::spawn_local(async move {
                let runner = Runner::new(window.clone()).await;
                window.request_redraw();
                tx.send(runner).expect("send runner");
            });
            #[cfg(not(target_arch = "wasm32"))]
            {
                let runner = futures::executor::block_on(Runner::new(window.clone()));
                window.request_redraw();
                tx.send(runner).expect("send runner");
            }
            rx
        };

        *self = BackgroundApp::Init { rx, window, runner };
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if matches!(self, Self::Running { .. }) {
            if matches!(event, winit::event::WindowEvent::RedrawRequested) {
                self.handle_commands(event_loop);
            }
        }

        if event_loop.exiting() {
            return;
        }

        match event {
            winit::event::WindowEvent::CloseRequested | winit::event::WindowEvent::Destroyed => {
                event_loop.exit();
            }
            winit::event::WindowEvent::Resized(size) => {
                if let Self::Running { runner, .. } = self {
                    runner.resize(size);
                }
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                if let Self::Running { input, .. } = self {
                    input.pos = position.into();
                }
            }
            winit::event::WindowEvent::MouseInput { state, .. } => {
                if let Self::Running { input, .. } = self {
                    if let winit::event::ElementState::Pressed = state {
                        input.force = Some(0.5);
                    }
                }
            }
            winit::event::WindowEvent::RedrawRequested => match self {
                Self::Init { window, runner, .. } => {
                    let window_clone = window.clone();

                    if let Ok(runner) = runner.try_recv() {
                        let Self::Init { rx, window, .. } = std::mem::take(self) else {
                            unreachable!()
                        };

                        let input = Input::new();

                        *self = Self::Running {
                            rx,
                            window: window.clone(),
                            canvas_id: None,
                            runner,
                            input,
                            frame_timer: chrono::Local::now(),
                            #[cfg(target_arch = "wasm32")]
                            redraw_closure: wasm_bindgen::prelude::Closure::wrap(Box::new(
                                move || {
                                    window.request_redraw();
                                },
                            )),
                        };
                    }

                    window_clone.request_redraw();
                }
                Self::Running {
                    runner,
                    window,
                    input,
                    frame_timer,
                    #[cfg(target_arch = "wasm32")]
                    redraw_closure,
                    ..
                } => {
                    const FPS: i64 = 30;
                    const FRAME_TIME: chrono::TimeDelta =
                        chrono::TimeDelta::nanoseconds(1_000_000_000 / FPS);

                    if runner.update(input) {
                        input.force = None;
                    }

                    // Schedule next redraw.
                    let elapsed = chrono::Local::now() - *frame_timer;
                    if elapsed < FRAME_TIME {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let sleep_time = FRAME_TIME - elapsed;
                            std::thread::sleep(std::time::Duration::from_nanos(
                                sleep_time.num_nanoseconds().map(|ns| ns as u64).unwrap_or(
                                    sleep_time
                                        .num_microseconds()
                                        .map(|us| us as u64 * 1_000)
                                        .unwrap_or(
                                            sleep_time.num_milliseconds() as u64 * 1_000_000,
                                        ),
                                ),
                            ));

                            window.request_redraw();
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            use wasm_bindgen::JsCast;
                            web_sys::window()
                                .and_then(|win| {
                                    win.set_timeout_with_callback_and_timeout_and_arguments_0(
                                        redraw_closure.as_ref().unchecked_ref(),
                                        (FRAME_TIME - elapsed).num_milliseconds() as i32,
                                    )
                                    .ok()
                                })
                                .expect("set timeout");
                        }
                    } else {
                        window.request_redraw();
                    }

                    *frame_timer = chrono::Local::now();
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl BackgroundApp {
    fn handle_commands(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Self::Running {
            rx,
            canvas_id,
            runner,
            window,
            input,
            ..
        } = self
        {
            for command in rx.try_iter() {
                match command {
                    BackgroundCommand::CreateCanvas(id) => {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            log::warn!("Create canvas called on native background: {id}");
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            use winit::platform::web::WindowExtWebSys;
                            web_sys::window()
                                .and_then(|win| win.document())
                                .and_then(|doc| {
                                    let container = doc.get_element_by_id(id)?;
                                    let canvas = web_sys::Element::from(window.canvas()?);
                                    container.append_child(&canvas).ok()?;
                                    Some(())
                                })
                                .expect("append canvas to background");

                            *canvas_id = Some(id);
                        }
                    }
                    BackgroundCommand::Resize(width, height) => {
                        window.set_min_inner_size(Some(LogicalSize::new(width, height)));
                        let _ = window.request_inner_size(LogicalSize::new(width, height));
                    }
                    BackgroundCommand::SetTheme(theme) => {
                        runner.set_theme(theme);
                    }
                    BackgroundCommand::Destroy => {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            log::warn!("Destroy called on native background");
                        }
                        #[cfg(target_arch = "wasm32")]
                        if let Some(id) = canvas_id {
                            web_sys::window()
                                .and_then(|win| win.document())
                                .and_then(|doc| {
                                    let container = doc.get_element_by_id(id)?;
                                    container.set_inner_html("");
                                    Some(())
                                })
                                .expect("append canvas to background");

                            event_loop.exit();
                        }
                    }
                    BackgroundCommand::PointerMove(x, y) => {
                        input.pos = (
                            x * window.scale_factor() as f32,
                            y * window.scale_factor() as f32,
                        );
                    }
                    BackgroundCommand::PointerDown(down) => {
                        input.force = Some(down);
                    }
                }
            }
        } else {
            log::warn!("Received command while not running");
        }
    }
}

#[derive(Debug)]
struct Input {
    pos: (f32, f32),
    force: Option<f32>,
}

impl Input {
    fn new() -> Self {
        Self {
            pos: (0.0, 0.0),
            force: None,
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Runner {
    gpu: Gpu,
    wave: Wave,
    postprocessor: PostProcessor,
    timer: chrono::DateTime<chrono::Local>,
}

impl Runner {
    async fn new(window: Arc<Window>) -> Self {
        let gpu = Gpu::new(window).await;
        let wave = Wave::new(&gpu.device, &gpu.queue, &gpu.config);
        let postprocessor = PostProcessor::new(
            &gpu.device,
            &gpu.queue,
            &gpu.config,
            Theme::Dark.background_color_hex(),
            &wave.textures,
        );
        let timer = chrono::Local::now();

        Self {
            gpu,
            wave,
            postprocessor,
            timer,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.gpu.resize(size);
            self.wave.resize(&self.gpu.device, &self.gpu.config);
            self.postprocessor
                .resize(&self.gpu.device, &self.wave.textures);
        }
    }

    fn set_theme(&mut self, theme: Theme) {
        self.wave.curr_theme = theme;
        self.postprocessor
            .update_base_color_buffer(&self.gpu.queue, theme.background_color_hex());
    }

    /// Returns whether the input should be cleared.
    fn update(&mut self, input: &Input) -> bool {
        const FPS: i64 = 30;
        const FRAME_TIME: chrono::TimeDelta = chrono::TimeDelta::nanoseconds(1_000_000_000 / FPS);

        if chrono::Local::now() - self.timer < FRAME_TIME {
            return false;
        } else {
            self.timer = chrono::Local::now();
        }

        self.wave
            .update(&self.gpu.queue, input.pos.0, input.pos.1, input.force);

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Runner Command Encoder"),
            });

        self.wave.render(&mut encoder);

        let texture = self
            .gpu
            .surface
            .get_current_texture()
            .expect("current texture");
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.gpu.config.view_formats[0]),
            ..Default::default()
        });

        self.postprocessor
            .render(&mut encoder, &view, self.wave.curr_wave);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        self.gpu.device.poll(wgpu::Maintain::Wait);
        texture.present();

        true
    }
}

#[derive(Debug)]
struct Gpu {
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,
}

impl Gpu {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: match cfg!(target_arch = "wasm32") {
                true => wgpu::Backends::GL,
                false => wgpu::Backends::PRIMARY,
            },
            ..Default::default()
        });

        let surface = instance.create_surface(window).expect("create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("request adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty() | wgpu::Features::FLOAT32_FILTERABLE,
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("request device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![surface_format.remove_srgb_suffix()],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Self {
            surface,
            queue,
            device,
            config,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct LogoTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl LogoTexture {
    const LIGHT_THEME: &'static [u8] = include_bytes!("../images/logo-light.png");
    const DARK_THEME: &'static [u8] = include_bytes!("../images/logo-dark.png");

    fn new(device: &wgpu::Device, queue: &wgpu::Queue, theme: Theme) -> Self {
        let data = match theme {
            Theme::Light => Self::LIGHT_THEME,
            Theme::Dark => Self::DARK_THEME,
        };

        let image = image::load_from_memory(data).expect("load image");
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Logo Texture"),
            size: wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::default(),
                aspect: wgpu::TextureAspect::default(),
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct WaveTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl WaveTexture {
    fn new(device: &wgpu::Device, width: NonZeroU32, height: NonZeroU32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Wave Texture"),
            size: wgpu::Extent3d {
                width: width.get(),
                height: height.get(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::R32Float],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }
}

#[derive(Debug)]
struct Wave {
    curr_wave: usize,
    curr_theme: Theme,
    action_buffer: wgpu::Buffer,
    textures: [WaveTexture; 3],
    logo_textures: [LogoTexture; 2],
    logo_sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: [wgpu::BindGroup; 6],
    pipeline: wgpu::RenderPipeline,
}

impl Wave {
    const BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Wave Bind Group Layout"),
            entries: &[
                // Action
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Wave at t - 2
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
                // Wave at t - 1
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Logo
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Logo sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

    const RESOLUTION_SCALE: f32 = 0.25;

    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let action_buffer = Self::create_action_buffer(device);
        let textures = Self::create_textures(device, config, Self::RESOLUTION_SCALE);
        let logo_textures = Self::create_logo_textures(device, queue);
        let logo_sampler = Self::create_logo_sampler(device);
        let bind_group_layout = device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT);
        let bind_groups = Self::create_bind_groups(
            device,
            &action_buffer,
            &textures,
            &logo_textures,
            &logo_sampler,
            &bind_group_layout,
        );
        let pipeline = Self::create_pipeline(device, &bind_group_layout);

        Self {
            curr_wave: 0,
            curr_theme: Theme::Dark,
            action_buffer,
            textures,
            logo_textures,
            logo_sampler,
            bind_group_layout,
            bind_groups,
            pipeline,
        }
    }

    fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.textures = Self::create_textures(device, config, Self::RESOLUTION_SCALE);
        self.bind_groups = Self::create_bind_groups(
            device,
            &self.action_buffer,
            &self.textures,
            &self.logo_textures,
            &self.logo_sampler,
            &self.bind_group_layout,
        );
    }

    fn update(&mut self, queue: &wgpu::Queue, x: f32, y: f32, force: Option<f32>) {
        let action = [
            x * Self::RESOLUTION_SCALE,
            y * Self::RESOLUTION_SCALE,
            force.unwrap_or(0.0),
            0.0,
        ];

        queue.write_buffer(&self.action_buffer, 0, bytemuck::cast_slice(&[action]));
    }

    fn render(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.curr_wave = (self.curr_wave + 1) % 3;

        let bind_group = &self.bind_groups[self.curr_wave * 2 + Self::theme_index(self.curr_theme)];

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Wave Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.textures[self.curr_wave].view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn theme_index(theme: Theme) -> usize {
        match theme {
            Theme::Light => 0,
            Theme::Dark => 1,
        }
    }

    fn create_action_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Wave Action Buffer"),
            size: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_textures(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        resolution_scale: f32,
    ) -> [WaveTexture; 3] {
        let texture_width =
            NonZeroU32::new(((config.width as f32 * resolution_scale) as u32).max(1))
                .expect("width");
        let texture_height =
            NonZeroU32::new(((config.height as f32 * resolution_scale) as u32).max(1))
                .expect("height");

        [
            WaveTexture::new(device, texture_width, texture_height),
            WaveTexture::new(device, texture_width, texture_height),
            WaveTexture::new(device, texture_width, texture_height),
        ]
    }

    fn create_logo_textures(device: &wgpu::Device, queue: &wgpu::Queue) -> [LogoTexture; 2] {
        [
            LogoTexture::new(device, queue, Theme::Light),
            LogoTexture::new(device, queue, Theme::Dark),
        ]
    }

    fn create_logo_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Logo Sampler"),
            ..Default::default()
        })
    }

    fn create_bind_groups(
        device: &wgpu::Device,
        action_buffer: &wgpu::Buffer,
        textures: &[WaveTexture; 3],
        logo_textures: &[LogoTexture; 2],
        logo_sampler: &wgpu::Sampler,
        layout: &wgpu::BindGroupLayout,
    ) -> [wgpu::BindGroup; 6] {
        std::array::from_fn(|index| {
            let i = index / 2;
            let j = index % 2;

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(
                    format!(
                        "Wave Bind Group {i} {}",
                        if j == 0 { "light" } else { "dark" }
                    )
                    .as_str(),
                ),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: action_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&textures[(i + 1) % 3].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&textures[(i + 2) % 3].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&logo_textures[j].view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(logo_sampler),
                    },
                ],
            })
        })
    }

    fn create_pipeline(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Wave Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/wave.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Wave Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Wave Pipeline"),
            layout: Some(&layout),
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
                    format: wgpu::TextureFormat::R32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }
}

#[derive(Debug)]
struct NoiseTexture {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl NoiseTexture {
    const SIZE: (u32, u32) = (512, 512);

    fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Noise Texture"),
            size: wgpu::Extent3d {
                width: Self::SIZE.0,
                height: Self::SIZE.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::R8Unorm],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self::render_noise(
            device,
            queue,
            &view,
            chrono::Local::now().timestamp() as u32,
        );

        Self { texture, view }
    }

    fn render_noise(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        seed: u32,
    ) {
        let seed_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Noise Seed Buffer"),
            contents: bytemuck::cast_slice(&[seed]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_gruop_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Noise Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Noise Bind Group"),
            layout: &bind_gruop_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: seed_buffer.as_entire_binding(),
            }],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Noise Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/noise_texture.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Noise Pipeline Layout"),
            bind_group_layouts: &[&bind_gruop_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Noise Pipeline"),
            layout: Some(&pipeline_layout),
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
                    format: wgpu::TextureFormat::R8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Noise Command Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Noise Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_pipeline(&pipeline);
            render_pass.draw(0..3, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
        device.poll(wgpu::Maintain::Wait);
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct PostProcessor {
    sampler: wgpu::Sampler,
    base_color_buffer: wgpu::Buffer,
    noise_texture: NoiseTexture,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: [wgpu::BindGroup; 3],
    pipeline: wgpu::RenderPipeline,
}

impl PostProcessor {
    const BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Processor Bind Group Layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        };

    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        base_color: u32,
        wave_textures: &[WaveTexture; 3],
    ) -> Self {
        let sampler = Self::create_sampler(device);
        let base_color_buffer = Self::create_base_color_buffer(device, base_color);
        let noise_texture = NoiseTexture::new(device, queue);
        let bind_group_layout = device.create_bind_group_layout(&Self::BIND_GROUP_LAYOUT);
        let bind_groups = Self::create_bind_groups(
            device,
            &bind_group_layout,
            wave_textures,
            &sampler,
            &base_color_buffer,
            &noise_texture,
        );
        let pipeline = Self::create_pipeline(device, &bind_group_layout, config.view_formats[0]);

        Self {
            base_color_buffer,
            sampler,
            noise_texture,
            bind_group_layout,
            bind_groups,
            pipeline,
        }
    }

    fn resize(&mut self, device: &wgpu::Device, wave_textures: &[WaveTexture; 3]) {
        self.bind_groups = Self::create_bind_groups(
            device,
            &self.bind_group_layout,
            wave_textures,
            &self.sampler,
            &self.base_color_buffer,
            &self.noise_texture,
        );
    }

    fn update_base_color_buffer(&mut self, queue: &wgpu::Queue, color: u32) {
        queue.write_buffer(&self.base_color_buffer, 0, bytemuck::cast_slice(&[color]));
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::TextureView,
        index: usize,
    ) {
        let bind_group = &self.bind_groups[index];

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post Processor Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: texture,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn create_base_color_buffer(device: &wgpu::Device, color: u32) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Post Processor Base Color Buffer"),
            contents: bytemuck::cast_slice(&[color]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Post Processor Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        })
    }

    fn create_bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        textures: &[WaveTexture; 3],
        sampler: &wgpu::Sampler,
        base_color_buffer: &wgpu::Buffer,
        noise_texture: &NoiseTexture,
    ) -> [wgpu::BindGroup; 3] {
        textures
            .iter()
            .enumerate()
            .map(|(i, texture)| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(format!("Post Processor Bind {i} Group").as_str()),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: base_color_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&noise_texture.view),
                        },
                    ],
                })
            })
            .collect_array()
            .expect("bind groups")
    }

    fn create_pipeline(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Post Processor Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/post_processor.wgsl").into()),
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Processor Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Post Processor Pipeline"),
            layout: Some(&layout),
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
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }
}
