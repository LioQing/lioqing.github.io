use wgpu::util::DeviceExt as _;

#[derive(Debug)]
pub struct DeltaTime {
    buffer: wgpu::Buffer,
}

impl DeltaTime {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Delta Time Buffer"),
            contents: bytemuck::bytes_of(&0.0f32),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self { buffer }
    }

    pub fn update(&self, queue: &wgpu::Queue, delta_time: f32) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&delta_time));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
