use glam::*;
use wgpu::util::DeviceExt as _;

use crate::meta_field::MetaField;

#[derive(Debug)]
pub struct QuadIndirect(wgpu::Buffer);

impl QuadIndirect {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Indirect Buffer"),
            contents: bytemuck::bytes_of(&wgpu::util::DrawIndirectArgs {
                vertex_count: 4,
                instance_count: 0,
                first_vertex: 0,
                first_instance: 0,
            }),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST,
        });

        Self(buffer)
    }

    pub fn reset(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.0,
            0,
            bytemuck::bytes_of(&wgpu::util::DrawIndirectArgs {
                vertex_count: 4,
                instance_count: 0,
                first_vertex: 0,
                first_instance: 0,
            }),
        );
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.0
    }
}

#[derive(Debug)]
pub struct Quads(wgpu::Buffer);

impl Quads {
    pub fn new(device: &wgpu::Device, meta_field: &MetaField) -> Self {
        let max_count = (meta_field.resolution().x - 1) * (meta_field.resolution().y - 1) * 8;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Segment Buffer"),
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
