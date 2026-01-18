use glam::*;
use itertools::Itertools as _;

use crate::{
    meta_shape::{MetaBox, MetaShapes},
    pipeline::RADIUS,
};

const HOVER_OFFSET: f32 = 8.0; // Has to match the CSS value
const SPRING_STIFFNESS: f32 = 100.0;
const SPRING_DAMPING: f32 = 10.0;

#[derive(Debug, Clone)]
pub struct Panel {
    pub element: web_sys::HtmlElement,
    pub top_left: Vec2,
    pub bottom_right: Vec2,
    pub curr_top_left: Vec2,
    pub curr_bottom_right: Vec2,
    pub vel_top_left: Vec2,
    pub vel_bottom_right: Vec2,
    pub hovered: bool,
}

impl Panel {
    pub fn hovered_top_left(&self) -> Vec2 {
        self.top_left - Vec2::splat(HOVER_OFFSET)
    }

    pub fn hovered_bottom_right(&self) -> Vec2 {
        self.bottom_right + Vec2::splat(HOVER_OFFSET)
    }

    pub fn target_top_left(&self) -> Vec2 {
        if self.hovered {
            self.hovered_top_left()
        } else {
            self.top_left
        }
    }

    pub fn target_bottom_right(&self) -> Vec2 {
        if self.hovered {
            self.hovered_bottom_right()
        } else {
            self.bottom_right
        }
    }

    pub fn needs_update(&self) -> bool {
        self.hovered != self.element.matches(":hover").unwrap_or(false)
            || self.curr_top_left != self.target_top_left()
            || self.curr_bottom_right != self.target_bottom_right()
    }

    fn spring_step(curr: Vec2, vel: Vec2, target: Vec2, delta_time: f32) -> (Vec2, Vec2) {
        if delta_time <= 0.0 {
            return (curr, vel);
        }

        let disp = curr - target;
        let accel = -SPRING_STIFFNESS * disp - SPRING_DAMPING * vel;
        let next_vel = vel + accel * delta_time;
        let next_pos = curr + next_vel * delta_time;
        (next_pos, next_vel)
    }
}

#[derive(Debug)]
pub struct PanelController {
    panels: Vec<Panel>,
}

impl PanelController {
    pub fn new(panels: impl IntoIterator<Item = web_sys::HtmlElement>, scroll_pos: IVec2) -> Self {
        Self {
            panels: panels
                .into_iter()
                .map(|element| {
                    let rect = element.get_bounding_client_rect();
                    Panel {
                        element,
                        top_left: Vec2::new(rect.left() as f32, rect.top() as f32)
                            + scroll_pos.as_vec2(),
                        bottom_right: Vec2::new(rect.right() as f32, rect.bottom() as f32)
                            + scroll_pos.as_vec2(),
                        curr_top_left: Vec2::new(rect.left() as f32, rect.top() as f32)
                            + scroll_pos.as_vec2(),
                        curr_bottom_right: Vec2::new(rect.right() as f32, rect.bottom() as f32)
                            + scroll_pos.as_vec2(),
                        vel_top_left: Vec2::ZERO,
                        vel_bottom_right: Vec2::ZERO,
                        hovered: false,
                    }
                })
                .collect(),
        }
    }

    pub fn resize(&mut self, meta_shapes: &mut MetaShapes, scroll_pos: IVec2) {
        for panel in self.panels.iter_mut() {
            let rect = panel.element.get_bounding_client_rect();
            panel.top_left =
                Vec2::new(rect.left() as f32, rect.top() as f32) + scroll_pos.as_vec2();
            panel.bottom_right =
                Vec2::new(rect.right() as f32, rect.bottom() as f32) + scroll_pos.as_vec2();
        }

        self.update_all_meta_boxes(meta_shapes, scroll_pos);
    }

    pub fn update(&mut self, meta_shapes: &mut MetaShapes, scroll_pos: IVec2, delta_time: f32) {
        let panels = self
            .panels
            .iter_mut()
            .enumerate()
            .filter(|(_, panel)| panel.needs_update())
            .update(|(_, panel)| {
                panel.hovered = panel.element.matches(":hover").unwrap_or(false);
                let target_top_left = panel.target_top_left();
                let target_bottom_right = panel.target_bottom_right();
                let (next_top_left, next_vel_top_left) = Panel::spring_step(
                    panel.curr_top_left,
                    panel.vel_top_left,
                    target_top_left,
                    delta_time,
                );
                let (next_bottom_right, next_vel_bottom_right) = Panel::spring_step(
                    panel.curr_bottom_right,
                    panel.vel_bottom_right,
                    target_bottom_right,
                    delta_time,
                );
                panel.curr_top_left = next_top_left;
                panel.curr_bottom_right = next_bottom_right;
                panel.vel_top_left = next_vel_top_left;
                panel.vel_bottom_right = next_vel_bottom_right;
            })
            .map(|(i, panel)| (i, &*panel));
        Self::update_meta_boxes(panels, meta_shapes, scroll_pos);
    }

    fn update_all_meta_boxes(&self, meta_shapes: &mut MetaShapes, scroll_pos: IVec2) {
        let panels = self.panels.iter().enumerate();
        Self::update_meta_boxes(panels, meta_shapes, scroll_pos);
    }

    fn update_meta_boxes<'a>(
        panels: impl IntoIterator<Item = (usize, &'a Panel)>,
        meta_shapes: &mut MetaShapes,
        scroll_pos: IVec2,
    ) {
        for (i, panel) in panels.into_iter() {
            meta_shapes.boxes_mut()[i] = MetaBox {
                min: panel.curr_top_left + Vec2::splat(RADIUS as f32),
                max: panel.curr_bottom_right - Vec2::splat(RADIUS as f32),
            };
        }
    }
}
