use glam::*;
use wgpu::util::DeviceExt as _;

use crate::frame::FrameMetadata;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetaFieldMetadata {
    offset: IVec2,
    cell_size: u32,
    fade_dist: u32,
}

#[derive(Debug)]
pub struct MetaField {
    resolution: UVec2,
    cell_size: u32,
    fade_dist: u32,
    offset: IVec2,
    buffer: wgpu::Buffer,
    texture: wgpu::Texture,
}

impl MetaField {
    pub fn new(
        device: &wgpu::Device,
        frame_metadata: &FrameMetadata,
        cell_size: u32,
        fade_dist: u32,
    ) -> Self {
        // We want to grid to be larger than the resolution,
        // so line segments' endpoints on the edge are still included.
        let resolution = frame_metadata.resolution() / cell_size + UVec2::splat(2);

        let offset = -(resolution * cell_size - frame_metadata.resolution()).as_ivec2() / 2;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meta Field Buffer"),
            contents: bytemuck::bytes_of(&MetaFieldMetadata {
                offset,
                cell_size,
                fade_dist,
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Meta Field Texture"),
            size: wgpu::Extent3d {
                width: resolution.x,
                height: resolution.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        Self {
            resolution,
            cell_size,
            fade_dist,
            offset,
            buffer,
            texture,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, frame_metadata: &FrameMetadata) {
        *self = Self::new(device, frame_metadata, self.cell_size, self.fade_dist);
    }

    pub fn resolution(&self) -> UVec2 {
        self.resolution
    }

    pub fn cell_size(&self) -> u32 {
        self.cell_size
    }

    pub fn fade_dist(&self) -> u32 {
        self.fade_dist
    }

    pub fn offset(&self) -> IVec2 {
        self.offset
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
