use std::sync::mpsc;

use glam::*;
use wasm_bindgen::{JsCast as _, UnwrapThrowExt as _};
use wasm_bindgen_futures::JsFuture;

use crate::{
    add_event_listener,
    ext::{DomRectExt as _, HtmlCollectionExt as _, WindowExt as _},
    frame::FrameMetadata,
    meta_shape::{MetaBox, MetaShapes},
    pipeline::{BackgroundImageRenderer, RADIUS},
};

const HOVER_OFFSET: f32 = 8.0; // Has to match the CSS value
const HOVER_OFFSET_SPLITTED: f32 = HOVER_OFFSET / 2.0; // For splitting between elevation and size

const SPRING_STIFFNESS: f32 = 500.0;
const SPRING_DAMPING: f32 = 25.0;

#[derive(Debug, Clone)]
pub enum PanelClass {
    // Hover on mouse over
    Interactive,
    // Follow size of HTML element
    Sized,
}

#[derive(Debug, Clone)]
pub struct ControlledPanel {
    pub class: PanelClass,
    pub curr_top_left_offset_vel: Vec2,
    pub curr_top_left_offset: Vec2,
    pub top_left_offset: Vec2,
    pub curr_bottom_right_offset_vel: Vec2,
    pub curr_bottom_right_offset: Vec2,
    pub bottom_right_offset: Vec2,
    pub curr_elevation_vel: f32,
    pub curr_elevation: f32,
    pub elevation: f32,
}

impl ControlledPanel {
    pub fn new_interactive() -> Self {
        Self {
            class: PanelClass::Interactive,
            ..Self::internal_default()
        }
    }

    pub fn new_sized() -> Self {
        Self {
            class: PanelClass::Sized,
            ..Self::internal_default()
        }
    }

    pub fn needs_offset_update(&self) -> bool {
        self.curr_elevation != self.elevation
            || self.curr_top_left_offset != self.top_left_offset
            || self.curr_bottom_right_offset != self.bottom_right_offset
    }

    /// Must override class field when using this
    fn internal_default() -> Self {
        Self {
            class: PanelClass::Interactive,
            curr_top_left_offset_vel: Vec2::ZERO,
            curr_top_left_offset: Vec2::ZERO,
            top_left_offset: Vec2::ZERO,
            curr_bottom_right_offset_vel: Vec2::ZERO,
            curr_bottom_right_offset: Vec2::ZERO,
            bottom_right_offset: Vec2::ZERO,
            curr_elevation_vel: 0.0,
            curr_elevation: 0.0,
            elevation: 0.0,
        }
    }
}

#[derive(Debug)]
pub enum PanelType {
    Controlled(ControlledPanel),
    Static,
}

impl PanelType {
    fn top_left_offset(&self) -> Vec2 {
        match self {
            PanelType::Controlled(panel) => panel.curr_top_left_offset,
            PanelType::Static => Vec2::ZERO,
        }
    }

    fn bottom_right_offset(&self) -> Vec2 {
        match self {
            PanelType::Controlled(panel) => panel.curr_bottom_right_offset,
            PanelType::Static => Vec2::ZERO,
        }
    }

    fn elevation(&self) -> f32 {
        match self {
            PanelType::Controlled(panel) => panel.curr_elevation,
            PanelType::Static => 0.0,
        }
    }
}

#[derive(Debug)]
pub struct Panel {
    pub element: web_sys::HtmlElement,
    pub panel_type: PanelType,
    pub top_left: Vec2,
    pub bottom_right: Vec2,
}

impl Panel {
    fn curr_top_left(&self) -> Vec2 {
        self.top_left + self.panel_type.top_left_offset()
    }

    fn curr_bottom_right(&self) -> Vec2 {
        self.bottom_right + self.panel_type.bottom_right_offset()
    }

    fn curr_elevation(&self) -> f32 {
        self.panel_type.elevation()
    }

    fn target_top_left(&self) -> Vec2 {
        self.top_left
            + match &self.panel_type {
                PanelType::Controlled(panel) => panel.top_left_offset,
                PanelType::Static => Vec2::ZERO,
            }
    }

    fn target_bottom_right(&self) -> Vec2 {
        self.bottom_right
            + match &self.panel_type {
                PanelType::Controlled(panel) => panel.bottom_right_offset,
                PanelType::Static => Vec2::ZERO,
            }
    }

    fn update(&mut self, delta_time: f32) -> bool {
        let PanelType::Controlled(panel) = &mut self.panel_type else {
            return false;
        };

        match panel.class {
            PanelClass::Interactive => {
                let rect = self.element.get_bounding_client_rect();
                let scroll_pos = web_sys::window().expect_throw("window").scroll_pos();
                let expected_top_left = rect.top_left() + scroll_pos.as_vec2();
                let expected_bottom_right = rect.bottom_right() + scroll_pos.as_vec2();

                let is_hovered = self.element.matches(":hover").unwrap_or(false);

                panel.elevation = if is_hovered {
                    HOVER_OFFSET_SPLITTED
                } else {
                    0.0
                };

                let hover_top_left_offset = if is_hovered {
                    Vec2::splat(-HOVER_OFFSET_SPLITTED)
                } else {
                    Vec2::ZERO
                };

                let hover_bottom_right_offset = if is_hovered {
                    Vec2::splat(HOVER_OFFSET_SPLITTED)
                } else {
                    Vec2::ZERO
                };

                panel.top_left_offset = expected_top_left - self.top_left + hover_top_left_offset;
                panel.bottom_right_offset =
                    expected_bottom_right - self.bottom_right + hover_bottom_right_offset;
            }
            PanelClass::Sized => {
                let rect = self.element.get_bounding_client_rect();
                let scroll_pos = web_sys::window().expect_throw("window").scroll_pos();
                let expected_top_left = rect.top_left() + scroll_pos.as_vec2();
                let expected_bottom_right = rect.bottom_right() + scroll_pos.as_vec2();

                panel.top_left_offset = expected_top_left - self.top_left;
                panel.bottom_right_offset = expected_bottom_right - self.bottom_right;
            }
        }

        if !panel.needs_offset_update() {
            return false;
        }

        // Spring simulation
        let displacement = panel.elevation - panel.curr_elevation;
        let spring_accel =
            SPRING_STIFFNESS * displacement - SPRING_DAMPING * panel.curr_elevation_vel;
        panel.curr_elevation_vel += spring_accel * delta_time;
        panel.curr_elevation += panel.curr_elevation_vel * delta_time;
        if panel.curr_elevation_vel.abs() < 1e-3 && displacement.abs() < 1e-3 {
            panel.curr_elevation = panel.elevation;
            panel.curr_elevation_vel = 0.0;
        }

        let displacement = panel.top_left_offset - panel.curr_top_left_offset;
        let spring_accel =
            SPRING_STIFFNESS * displacement - SPRING_DAMPING * panel.curr_top_left_offset_vel;
        panel.curr_top_left_offset_vel += spring_accel * delta_time;
        panel.curr_top_left_offset += panel.curr_top_left_offset_vel * delta_time;
        if panel.curr_top_left_offset_vel.length_squared() < 1e-6
            && displacement.length_squared() < 1e-6
        {
            panel.curr_top_left_offset = panel.top_left_offset;
            panel.curr_top_left_offset_vel = Vec2::ZERO;
        }

        let displacement = panel.bottom_right_offset - panel.curr_bottom_right_offset;
        let spring_accel =
            SPRING_STIFFNESS * displacement - SPRING_DAMPING * panel.curr_bottom_right_offset_vel;
        panel.curr_bottom_right_offset_vel += spring_accel * delta_time;
        panel.curr_bottom_right_offset += panel.curr_bottom_right_offset_vel * delta_time;
        if panel.curr_bottom_right_offset_vel.length_squared() < 1e-6
            && displacement.length_squared() < 1e-6
        {
            panel.curr_bottom_right_offset = panel.bottom_right_offset;
            panel.curr_bottom_right_offset_vel = Vec2::ZERO;
        }

        self.clip();

        true
    }

    pub fn clip(&mut self) {
        if let PanelType::Controlled(ControlledPanel {
            class: PanelClass::Sized,
            ..
        }) = self.panel_type
        {
            // Hide everything outside of the current panel size using clip-path
            let clip_path = format!(
                "xywh({}px {}px {}px {}px)",
                self.curr_top_left().x - self.target_top_left().x,
                self.curr_top_left().y - self.target_top_left().y,
                self.curr_bottom_right().x - self.curr_top_left().x,
                self.curr_bottom_right().y - self.curr_top_left().y,
            );

            if let Err(e) = self.element.style().set_property("clip-path", &clip_path) {
                log::error!(
                    "Failed to set clip-path for panel: {}",
                    e.as_string().unwrap_or("Unknown error".to_string())
                );
            }
        }
    }
}

#[derive(Debug)]
pub struct PanelController {
    panels: Vec<Panel>,
}

impl PanelController {
    pub fn new() -> Self {
        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let scroll_pos = window.scroll_pos();

        let panels = document
            .get_elements_by_class_name("panel")
            .iter()
            .map(|el| el.dyn_into::<web_sys::HtmlElement>().unwrap_throw())
            .collect::<Vec<_>>();

        Self {
            panels: panels
                .into_iter()
                .map(|element| {
                    let rect = element.get_bounding_client_rect();
                    let panel_type = if element.class_list().contains("interactive-panel") {
                        PanelType::Controlled(ControlledPanel::new_interactive())
                    } else if element.class_list().contains("sized-panel") {
                        PanelType::Controlled(ControlledPanel::new_sized())
                    } else {
                        PanelType::Static
                    };

                    Panel {
                        top_left: Vec2::new(rect.left() as f32, rect.top() as f32)
                            + scroll_pos.as_vec2(),
                        bottom_right: Vec2::new(rect.right() as f32, rect.bottom() as f32)
                            + scroll_pos.as_vec2(),
                        element,
                        panel_type,
                    }
                })
                .collect(),
        }
    }

    pub fn update(&mut self, meta_shapes: &mut MetaShapes, delta_time: f32) {
        let panels = self
            .panels
            .iter_mut()
            .enumerate()
            .filter_map(|(i, panel)| panel.update(delta_time).then_some((i, &*panel)));
        Self::update_meta_boxes(panels, meta_shapes);
    }

    pub fn resize(&mut self, meta_shapes: &mut MetaShapes, scroll_pos: IVec2) {
        for panel in self.panels.iter_mut() {
            let rect = panel.element.get_bounding_client_rect();
            panel.top_left =
                Vec2::new(rect.left() as f32, rect.top() as f32) + scroll_pos.as_vec2();
            panel.bottom_right =
                Vec2::new(rect.right() as f32, rect.bottom() as f32) + scroll_pos.as_vec2();
            panel.clip();
        }

        self.update_all_meta_boxes(meta_shapes);
    }

    pub fn panel_count(&self) -> usize {
        self.panels.len()
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

#[derive(Debug)]
pub struct BackgroundController {
    project_element: web_sys::HtmlElement,
    position: Vec2,
}

impl BackgroundController {
    pub fn new() -> Self {
        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let project_element = document
            .get_element_by_id("projects")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let project_rect = project_element.get_bounding_client_rect();

        Self {
            project_element,
            position: Vec2::new(0.0, project_rect.top() as f32),
        }
    }

    pub fn update(&mut self, frame_metadata: &FrameMetadata) {
        let project_rect = self.project_element.get_bounding_client_rect();

        self.position = Vec2::new(
            0.0,
            project_rect.top() as f32 + frame_metadata.top_left().y as f32 * 0.2,
        );
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }
}

#[derive(Debug)]
pub struct SkillsController {
    pub element: web_sys::HtmlElement,
    pub top_left: Vec2,
    pub bottom_right: Vec2,
}

impl SkillsController {
    pub fn new() -> Self {
        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let element = document
            .get_element_by_id("skills")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let rect = element.get_bounding_client_rect();

        Self {
            element,
            top_left: rect.top_left(),
            bottom_right: rect.bottom_right(),
        }
    }

    pub fn update(&mut self) {
        let rect = self.element.get_bounding_client_rect();
        self.top_left = rect.top_left();
        self.bottom_right = rect.bottom_right();
    }

    pub fn top_left(&self) -> Vec2 {
        self.top_left
    }

    pub fn bottom_right(&self) -> Vec2 {
        self.bottom_right
    }
}
