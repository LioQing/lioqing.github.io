use glam::*;
use wgpu::util::DeviceExt as _;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetaImageMetadata {
    min: Vec2,
    max: Vec2,
    multiplier: f32,
    padding: u32,
}

#[derive(Debug)]
pub struct MetaImage {
    metadata: MetaImageMetadata,
    buffer: wgpu::Buffer,
    texture: wgpu::Texture,
}

impl MetaImage {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        min: Vec2,
        max: Vec2,
        multiplier: f32,
        image: &image::DynamicImage,
    ) -> Self {
        let metadata = MetaImageMetadata {
            min,
            max,
            multiplier,
            ..Default::default()
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meta Image Metadata Buffer"),
            contents: bytemuck::bytes_of(&metadata),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Meta Image Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::default(),
            &data,
        );

        Self {
            metadata,
            buffer,
            texture,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, min: Vec2, max: Vec2, multiplier: f32) {
        self.metadata = MetaImageMetadata {
            min,
            max,
            multiplier,
            ..self.metadata
        };

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.metadata));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}
