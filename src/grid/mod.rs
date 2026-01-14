use glam::*;

use crate::frame::FrameMetadata;

use wgpu::util::DeviceExt;

pub mod pipeline;
pub mod target;

pub const GRID_CELL_SIZE: u32 = 48;

#[derive(Debug)]
pub struct GridMetadata(FrameMetadata);

impl GridMetadata {
    pub fn new(device: &wgpu::Device, frame_metadata: &FrameMetadata) -> Self {
        Self(FrameMetadata::new(
            device,
            frame_metadata.resolution() / GRID_CELL_SIZE + UVec2::splat(2),
            -IVec2::splat(GRID_CELL_SIZE as i32) / 2,
        ))
    }

    pub fn update(&mut self, queue: &wgpu::Queue, frame_metadata: &FrameMetadata) {
        self.0.update(
            queue,
            frame_metadata.resolution() / GRID_CELL_SIZE + UVec2::splat(2),
            -IVec2::splat(GRID_CELL_SIZE as i32) / 2,
        );
    }

    pub fn resolution(&self) -> UVec2 {
        self.0.resolution()
    }

    pub fn top_left(&self) -> IVec2 {
        self.0.top_left()
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.0.buffer()
    }
}

#[derive(Debug)]
pub struct GridState {
    pos: wgpu::Buffer,
    vel: wgpu::Buffer,
}

impl GridState {
    pub fn new(device: &wgpu::Device, grid_metadata: &GridMetadata) -> Self {
        let len = (2 * grid_metadata.resolution().x * grid_metadata.resolution().y) as usize;
        let zeros = vec![Vec2::ZERO; len];

        let pos = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid State Buffer"),
            contents: bytemuck::cast_slice(&zeros),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let vel = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Velocity Buffer"),
            contents: bytemuck::cast_slice(&zeros),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self { pos, vel }
    }

    pub fn resize(&mut self, device: &wgpu::Device, grid_metadata: &GridMetadata) {
        let len = (2 * grid_metadata.resolution().x * grid_metadata.resolution().y) as usize;
        let zeros = vec![Vec2::ZERO; len];

        self.pos = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid State Buffer"),
            contents: bytemuck::cast_slice(&zeros),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        self.vel = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Velocity Buffer"),
            contents: bytemuck::cast_slice(&zeros),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
    }

    pub fn pos_buffer(&self) -> &wgpu::Buffer {
        &self.pos
    }

    pub fn vel_buffer(&self) -> &wgpu::Buffer {
        &self.vel
    }
}
