use crate::mar_sq::{
    line_segment::{LineSegmentIndirect, LineSegments},
    quad::{QuadIndirect, Quads},
};

pub trait MarchingSquaresShape {
    type Indirect: MarchingSquaresShapeIndirect;
    type Buffer: MarchingSquaresShapeBuffer;
    const PREPROCESS_SHADER: &'static str;
    const RENDER_SHADER: &'static str;
}

impl MarchingSquaresShape for LineSegments {
    type Indirect = LineSegmentIndirect;
    type Buffer = LineSegments;
    const PREPROCESS_SHADER: &'static str =
        include_str!("../shader/line_segment_marching_squares.wgsl");
    const RENDER_SHADER: &'static str = include_str!("../shader/line_segment.wgsl");
}

impl MarchingSquaresShape for Quads {
    type Indirect = QuadIndirect;
    type Buffer = Quads;
    const PREPROCESS_SHADER: &'static str = include_str!("../shader/quad_marching_squares.wgsl");
    const RENDER_SHADER: &'static str = include_str!("../shader/quad.wgsl");
}

pub trait MarchingSquaresShapeIndirect {
    fn new(device: &wgpu::Device) -> Self;
    fn buffer(&self) -> &wgpu::Buffer;
    fn reset(&self, queue: &wgpu::Queue);
}

impl MarchingSquaresShapeIndirect for LineSegmentIndirect {
    fn new(device: &wgpu::Device) -> Self {
        LineSegmentIndirect::new(device)
    }

    fn buffer(&self) -> &wgpu::Buffer {
        self.buffer()
    }

    fn reset(&self, queue: &wgpu::Queue) {
        self.reset(queue)
    }
}

impl MarchingSquaresShapeIndirect for QuadIndirect {
    fn new(device: &wgpu::Device) -> Self {
        QuadIndirect::new(device)
    }

    fn buffer(&self) -> &wgpu::Buffer {
        self.buffer()
    }

    fn reset(&self, queue: &wgpu::Queue) {
        self.reset(queue)
    }
}

pub trait MarchingSquaresShapeBuffer {
    fn buffer(&self) -> &wgpu::Buffer;
}

impl MarchingSquaresShapeBuffer for LineSegments {
    fn buffer(&self) -> &wgpu::Buffer {
        self.buffer()
    }
}

impl MarchingSquaresShapeBuffer for Quads {
    fn buffer(&self) -> &wgpu::Buffer {
        self.buffer()
    }
}
