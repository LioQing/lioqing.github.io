use glam::*;
use itertools::Itertools as _;

use crate::{
    meta_shape::{MetaBox, MetaShapes},
    pipeline::RADIUS,
};

const HOVER_OFFSET: f32 = 8.0; // Has to match the CSS value
const HOVER_OFFSET_SPLITTED: f32 = HOVER_OFFSET / 2.0; // For splitting between elevation and size
const SPRING_STIFFNESS: f32 = 80.0;
const SPRING_DAMPING: f32 = 8.0;

#[derive(Debug, Clone)]
struct Panel {
    pub element: web_sys::HtmlElement,
    pub top_left: Vec2,
    pub bottom_right: Vec2,
    pub hovered: bool,
    pub hover_progress: f32,
    pub hover_progress_vel: f32,
}

impl Panel {
    fn needs_update(&self) -> bool {
        self.hovered != self.element.matches(":hover").unwrap_or(false)
            || (self.hovered && self.hover_progress != 1.0)
            || (!self.hovered && self.hover_progress != 0.0)
    }

    fn curr_top_left(&self) -> Vec2 {
        self.top_left - Vec2::splat(HOVER_OFFSET_SPLITTED) * self.hover_progress
    }

    fn curr_bottom_right(&self) -> Vec2 {
        self.bottom_right + Vec2::splat(HOVER_OFFSET_SPLITTED) * self.hover_progress
    }

    fn curr_elevation(&self) -> f32 {
        self.hover_progress * HOVER_OFFSET_SPLITTED
    }

    fn update(&mut self, delta_time: f32) {
        let target = if self.element.matches(":hover").unwrap_or(false) {
            1.0
        } else {
            0.0
        };
        let displacement = target - self.hover_progress;
        let spring_accel =
            SPRING_STIFFNESS * displacement - SPRING_DAMPING * self.hover_progress_vel;
        self.hover_progress_vel += spring_accel * delta_time;
        self.hover_progress += self.hover_progress_vel * delta_time;

        // Clamp
        if (self.hover_progress - target).abs() < 0.001 && self.hover_progress_vel.abs() < 0.001 {
            self.hover_progress = target;
            self.hover_progress_vel = 0.0;
        }
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
                        hovered: false,
                        hover_progress: 0.0,
                        hover_progress_vel: 0.0,
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

        self.update_all_meta_boxes(meta_shapes);
    }

    pub fn update(&mut self, meta_shapes: &mut MetaShapes, delta_time: f32) {
        let panels = self
            .panels
            .iter_mut()
            .enumerate()
            .filter(|(_, panel)| panel.needs_update())
            .update(|(_, panel)| {
                panel.update(delta_time);
            })
            .map(|(i, panel)| (i, &*panel));
        Self::update_meta_boxes(panels, meta_shapes);
    }

    fn update_all_meta_boxes(&self, meta_shapes: &mut MetaShapes) {
        let panels = self.panels.iter().enumerate();
        Self::update_meta_boxes(panels, meta_shapes);
    }

    fn update_meta_boxes<'a>(
        panels: impl IntoIterator<Item = (usize, &'a Panel)>,
        meta_shapes: &mut MetaShapes,
    ) {
        for (i, panel) in panels.into_iter() {
            meta_shapes.boxes_mut()[i] = MetaBox {
                min: panel.curr_top_left() + Vec2::splat(RADIUS as f32),
                max: panel.curr_bottom_right() - Vec2::splat(RADIUS as f32),
                elevation: panel.curr_elevation(),
                ..Default::default()
            };
        }
    }
}
