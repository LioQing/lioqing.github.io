use glam::*;

use crate::meta_field::MetaField;

#[derive(Debug)]
pub struct LineSegments(wgpu::Buffer);

impl LineSegments {
    pub fn new(device: &wgpu::Device, meta_field: &MetaField) -> Self {
        let max_count = (meta_field.resolution().x - 1) * (meta_field.resolution().y - 1) * 2;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Line Segment Buffer"),
            size: (std::mem::size_of::<IVec4>() * max_count as usize) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self(buffer)
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.0
    }
}
