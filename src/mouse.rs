use glam::*;

use crate::frame::FrameMetadata;

#[derive(Debug)]
pub struct Mouse {
    target: Vec2,
    position: Vec2,
    hidden: bool,
}

impl Mouse {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            target: position,
            hidden: true,
        }
    }

    pub fn update(&mut self, frame_metadata: &FrameMetadata, delta_time: f32) {
        let global_target = self.target + frame_metadata.top_left().as_vec2();

        const FACTOR: f32 = 20.0;
        self.position = self.position.lerp(global_target, delta_time * FACTOR);

        const EPSILON: f32 = 0.001;
        if (self.position - global_target).length_squared() < EPSILON {
            self.position = global_target;
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn target(&self) -> Vec2 {
        self.target
    }

    pub fn set_target(&mut self, target: Vec2) {
        self.target = target;
        self.hidden = false;
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }
}
