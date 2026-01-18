use glam::*;
use wgpu::util::DeviceExt as _;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameMetadataPod {
    pub resolution: UVec2,
    pub top_left: IVec2,
}

#[derive(Debug)]
pub struct FrameMetadata {
    resolution: UVec2,
    top_left: IVec2,
    buffer: wgpu::Buffer,
}

impl FrameMetadata {
    pub fn new(device: &wgpu::Device, resolution: UVec2, top_left: IVec2) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame Metadata Buffer"),
            contents: bytemuck::bytes_of(&FrameMetadataPod {
                resolution,
                top_left,
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            resolution,
            top_left,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, resolution: UVec2, top_left: IVec2) {
        self.resolution = resolution;
        self.top_left = top_left;

        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::bytes_of(&FrameMetadataPod {
                resolution,
                top_left,
            }),
        );
    }

    pub fn resolution(&self) -> UVec2 {
        self.resolution
    }

    pub fn top_left(&self) -> IVec2 {
        self.top_left
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn center(&self) -> Vec2 {
        self.top_left.as_vec2() + self.resolution.as_vec2() * 0.5
    }
}
