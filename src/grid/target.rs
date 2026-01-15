use glam::*;
use wgpu::util::DeviceExt as _;

#[derive(Debug)]
pub struct Target {
    position: Vec2,
    velocity: Vec2,
    buffer: wgpu::Buffer,
}

impl Target {
    pub fn new(device: &wgpu::Device, init_target: Vec2) -> Self {
        let position = init_target;
        let velocity = Vec2::ZERO;

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Target Buffer"),
            contents: bytemuck::bytes_of(&position),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            position,
            velocity,
            buffer,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, new_target: Vec2, delta_time: f32) {
        const SPRING_K: f32 = 80.0;
        const DAMPING: f32 = 8.0;

        if delta_time > 0.0 {
            let disp = self.position - new_target;
            let accel = -SPRING_K * disp - DAMPING * self.velocity;
            self.velocity += accel * delta_time;
            self.position += self.velocity * delta_time;

            const POS_EPS_SQ: f32 = 0.01;
            const VEL_EPS_SQ: f32 = 0.01;
            if (self.position - new_target).length_squared() < POS_EPS_SQ
                && self.velocity.length_squared() < VEL_EPS_SQ
            {
                self.position = new_target;
                self.velocity = Vec2::ZERO;
            }
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.position));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
