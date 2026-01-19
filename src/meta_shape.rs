use glam::*;
use wgpu::util::DeviceExt as _;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetaBall {
    pub center: Vec2,
    pub radius: f32,
    pub _padding: f32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetaLine {
    pub start: Vec2,
    pub end: Vec2,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetaBox {
    pub min: Vec2,
    pub max: Vec2,
    pub elevation: f32,
    pub _padding: f32,
}

#[derive(Debug)]
pub struct MetaShapes {
    balls: Vec<MetaBall>,
    boxes: Vec<MetaBox>,
    balls_buffer: wgpu::Buffer,
    boxes_buffer: wgpu::Buffer,
}

impl MetaShapes {
    pub fn new(device: &wgpu::Device, ball_count: usize, box_count: usize) -> Self {
        let balls = vec![MetaBall::default(); ball_count];
        let boxes = vec![MetaBox::default(); box_count];
        let balls_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meta Balls Buffer"),
            contents: bytemuck::cast_slice(&balls),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let boxes_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meta Boxes Buffer"),
            contents: bytemuck::cast_slice(&boxes),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            balls,
            boxes,
            balls_buffer,
            boxes_buffer,
        }
    }

    pub fn balls(&self) -> &[MetaBall] {
        &self.balls
    }

    pub fn boxes(&self) -> &[MetaBox] {
        &self.boxes
    }

    pub fn balls_mut(&mut self) -> &mut [MetaBall] {
        &mut self.balls
    }

    pub fn boxes_mut(&mut self) -> &mut [MetaBox] {
        &mut self.boxes
    }

    pub fn ensure_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.balls_buffer, 0, bytemuck::cast_slice(&self.balls));
        queue.write_buffer(&self.boxes_buffer, 0, bytemuck::cast_slice(&self.boxes));
    }

    pub fn balls_buffer(&self) -> &wgpu::Buffer {
        &self.balls_buffer
    }

    pub fn boxes_buffer(&self) -> &wgpu::Buffer {
        &self.boxes_buffer
    }
}
