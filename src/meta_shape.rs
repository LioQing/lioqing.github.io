use glam::*;
use wgpu::util::DeviceExt as _;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
struct MetaShapesMetadata {
    ball_count: u32,
    line_count: u32,
    box_count: u32,
    _padding: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetaBall {
    pub position: Vec2,
    pub radius: f32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetaLine {
    pub start: Vec2,
    pub end: Vec2,
    pub radius: f32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MetaBox {
    pub min: Vec2,
    pub max: Vec2,
    pub radius: f32,
}

#[derive(Debug)]
pub struct MetaShapes {
    balls: Vec<MetaBall>,
    lines: Vec<MetaLine>,
    boxes: Vec<MetaBox>,
    buffer: wgpu::Buffer,
}

impl MetaShapes {
    pub fn new(
        device: &wgpu::Device,
        ball_count: usize,
        line_count: usize,
        box_count: usize,
    ) -> Self {
        let balls = vec![MetaBall::default(); ball_count];
        let lines = vec![MetaLine::default(); line_count];
        let boxes = vec![MetaBox::default(); box_count];
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Meta Shapes Buffer"),
            contents: [
                bytemuck::bytes_of(&MetaShapesMetadata {
                    ball_count: ball_count as u32,
                    line_count: line_count as u32,
                    box_count: box_count as u32,
                    ..Default::default()
                }),
                bytemuck::cast_slice(&balls),
                bytemuck::cast_slice(&lines),
                bytemuck::cast_slice(&boxes),
            ]
            .concat()
            .as_slice(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            balls,
            lines,
            boxes,
            buffer,
        }
    }

    pub fn balls(&self) -> &[MetaBall] {
        &self.balls
    }

    pub fn lines(&self) -> &[MetaLine] {
        &self.lines
    }

    pub fn boxes(&self) -> &[MetaBox] {
        &self.boxes
    }

    pub fn balls_mut(&mut self) -> &mut [MetaBall] {
        &mut self.balls
    }

    pub fn lines_mut(&mut self) -> &mut [MetaLine] {
        &mut self.lines
    }

    pub fn boxes_mut(&mut self) -> &mut [MetaBox] {
        &mut self.boxes
    }

    pub fn ensure_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.buffer,
            std::mem::size_of::<MetaShapesMetadata>() as wgpu::BufferAddress,
            bytemuck::cast_slice(&self.balls),
        );
        queue.write_buffer(
            &self.buffer,
            (std::mem::size_of::<MetaShapesMetadata>()
                + std::mem::size_of::<MetaBall>() * self.balls.len())
                as wgpu::BufferAddress,
            bytemuck::cast_slice(&self.lines),
        );
        queue.write_buffer(
            &self.buffer,
            (std::mem::size_of::<MetaShapesMetadata>()
                + std::mem::size_of::<MetaBall>() * self.balls.len()
                + std::mem::size_of::<MetaLine>() * self.lines.len())
                as wgpu::BufferAddress,
            bytemuck::cast_slice(&self.boxes),
        );
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
