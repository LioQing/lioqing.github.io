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
    about_me_element: web_sys::HtmlElement,
    skills_element: web_sys::HtmlElement,
    zero_one_position: Vec2,
    skills_position: Vec2,
}

impl BackgroundController {
    pub fn new() -> Self {
        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let about_me_element = document
            .get_element_by_id("about-me")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let skills_element = document
            .get_element_by_id("skills")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let about_me_rect = about_me_element.get_bounding_client_rect();
        let skills_rect = skills_element.get_bounding_client_rect();

        Self {
            about_me_element,
            skills_element,
            zero_one_position: Vec2::new(0.0, about_me_rect.bottom() as f32),
            skills_position: Vec2::new(0.0, skills_rect.top() as f32),
        }
    }

    pub fn update(
        &mut self,
        frame_metadata: &FrameMetadata,
        zero_one_renderer: &BackgroundImageRenderer,
    ) {
        let about_me_rect = self.about_me_element.get_bounding_client_rect();
        let skills_rect = self.skills_element.get_bounding_client_rect();

        self.zero_one_position = Vec2::new(
            0.0,
            about_me_rect.bottom() as f32 + frame_metadata.top_left().y as f32 * 0.2,
        );
        self.skills_position = Vec2::new(
            0.0,
            skills_rect.top() as f32 + frame_metadata.top_left().y as f32 * 0.2,
        );

        if self.skills_position.y - self.zero_one_position.y
            < zero_one_renderer.get_size(frame_metadata).y as f32
        {
            self.skills_position.y =
                self.zero_one_position.y + zero_one_renderer.get_size(frame_metadata).y as f32
        }
    }

    pub fn zero_one_position(&self) -> Vec2 {
        self.zero_one_position
    }

    pub fn skills_position(&self) -> Vec2 {
        self.skills_position
    }
}

#[derive(Debug)]
pub struct ProjectController;

#[derive(Debug, Clone, serde::Deserialize)]
struct ProjectLink {
    label: String,
    url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ProjectData {
    title: String,
    tags: Vec<String>,
    year: u32,
    month: u32,
    subtitle: String,
    description: String,
    image_url: String,
    links: Vec<ProjectLink>,
}

impl ProjectController {
    pub async fn new() -> Self {
        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let element = document
            .get_element_by_id("projects")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let response = JsFuture::from(window.fetch_with_str("./assets/projects.json"))
            .await
            .unwrap_throw()
            .dyn_into::<web_sys::Response>()
            .unwrap_throw();
        let json = JsFuture::from(response.json().unwrap_throw())
            .await
            .unwrap_throw();
        let projects = serde_wasm_bindgen::from_value::<Vec<ProjectData>>(json).unwrap_throw();

        let html = Self::render_project_list(&projects);
        element.set_inner_html(&html);

        let details = (0..projects.len())
            .map(|i| {
                document
                    .get_element_by_id(&format!("project-details-{i}"))
                    .unwrap_throw()
                    .dyn_into::<web_sys::HtmlElement>()
                    .unwrap_throw()
            })
            .collect::<Vec<_>>();

        let items = (0..projects.len())
            .map(|i| {
                document
                    .get_element_by_id(&format!("project-item-{i}"))
                    .unwrap_throw()
                    .dyn_into::<web_sys::HtmlElement>()
                    .unwrap_throw()
            })
            .collect::<Vec<_>>();

        for (i, item) in items.into_iter().enumerate() {
            add_event_listener!(item, "click", {
                let details = details.clone();
                move |_event: web_sys::Event| {
                    for (idx, detail) in details.iter().enumerate() {
                        if idx == i {
                            let is_expanded = detail
                                .get_attribute("data-expanded")
                                .as_deref()
                                == Some("true");
                            ProjectController::set_project_details_expanded(
                                detail,
                                !is_expanded,
                            );
                        } else {
                            ProjectController::set_project_details_expanded(detail, false);
                        }
                    }
                }
            }; FnMut(_));
        }

        Self
    }

    fn render_project_list(projects: &[ProjectData]) -> String {
        let items = projects
            .iter()
            .enumerate()
            .map(|(i, project)| Self::render_project_list_item(i, project))
            .collect::<String>();

        format!(
            "
            <div style=\"
                display: flex;
                flex-direction: column;
                gap: 36px;
            \">
                {items}
            </div>
            "
        )
    }

    fn render_project_list_item(index: usize, project: &ProjectData) -> String {
        let tags_html = Self::render_project_tags(&project.tags);
        let date = format!("{} {}", show_month(project.month), project.year);
        let details_html = Self::render_project_details(project);

        format!(
            "
            <div
                class=\"panel sized-panel\"
                style=\"display: flex; flex-direction: column; width: 100%;\"
            >
                <div
                    id=\"project-item-{index}\"
                    style=\"cursor: pointer; padding: 24px 36px;\"
                >
                    <p>{date}</p>
                    <h3>{}</h3>
                    <p>{}</p>
                    <div style=\"
                        display: flex;
                        flex-wrap: wrap;
                        gap: 8px;
                        margin-top: 6px;
                    \">{tags_html}</div>
                </div>
                <div
                    id=\"project-details-{index}\"
                    data-expanded=\"false\"
                    style=\"
                        height: 0px;
                        opacity: 0;
                        overflow: hidden;
                        margin-top: 0px;
                        transition: ease-in-out 0.3s all;
                    \"
                >
                    {details_html}
                </div>
            </div>
            ",
            project.title, project.subtitle
        )
    }

    fn render_project_details(project: &ProjectData) -> String {
        let links_html = Self::render_project_links(&project.links);

        format!(
            "
            <div style=\"
                padding: 0px 36px 36px 36px;
                display: flex;
                flex-direction: row;
                gap: 24px;
                flex-wrap: wrap;
            \">
                <img src=\"assets/projects/{}\" alt=\"{}\" style=\"
                    width: max(480px, 40%);
                    height: auto;
                    border-radius: 12px;
                \" />
                <div style=\"
                    flex: 1;
                    display: flex;
                    flex-direction: column;
                    gap: 12px;
                    min-width: 200px;
                \">
                    <p>{}</p>
                    <div style=\"
                        display: flex;
                        flex-wrap: wrap;
                        gap: 12px;
                    \">{links_html}</div>
                </div>
            </div>
            ",
            project.image_url, project.title, project.description
        )
    }

    fn set_project_details_expanded(detail: &web_sys::HtmlElement, expanded: bool) {
        let height = if expanded {
            format!("{}px", detail.scroll_height())
        } else {
            "0px".to_string()
        };
        let opacity = if expanded { "1" } else { "0" };
        let margin_top = if expanded { "12px" } else { "0px" };

        if let Err(e) = detail.style().set_property("height", &height) {
            log::error!(
                "Failed to update project details height: {}",
                e.as_string().unwrap_or("Unknown error".to_string())
            );
        }
        if let Err(e) = detail.style().set_property("opacity", opacity) {
            log::error!(
                "Failed to update project details opacity: {}",
                e.as_string().unwrap_or("Unknown error".to_string())
            );
        }
        if let Err(e) = detail.style().set_property("margin-top", margin_top) {
            log::error!(
                "Failed to update project details margin: {}",
                e.as_string().unwrap_or("Unknown error".to_string())
            );
        }
        if let Err(e) =
            detail.set_attribute("data-expanded", if expanded { "true" } else { "false" })
        {
            log::error!(
                "Failed to update project details state: {}",
                e.as_string().unwrap_or("Unknown error".to_string())
            );
        }
    }

    fn render_project_tags(tags: &[String]) -> String {
        tags.iter()
            .map(|tag| {
                format!(
                    "<span style=\"
                        padding: 4px 8px;
                        border-radius: 999px;
                        background: rgba(255, 255, 255, 0.08);
                        font-size: 0.85em;
                    \">{}</span>",
                    tag
                )
            })
            .collect::<String>()
    }

    fn render_project_links(links: &[ProjectLink]) -> String {
        links
            .iter()
            .map(|link| {
                format!(
                    "<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">{}</a>",
                    link.url, link.label
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, serde::Deserialize)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum ExperienceFilter {
    #[default]
    Work,
    Education,
    Others,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct ExperienceData {
    name: String,
    organization: String,
    filter: ExperienceFilter,
    start_year: u32,
    start_month: u32,
    end_year: Option<u32>,
    end_month: Option<u32>,
}

#[derive(Debug)]
struct Experience {
    data: ExperienceData,
    element: web_sys::HtmlElement,
    height: f32,
}

#[derive(Debug)]
pub struct ExperienceController {
    experiences: Vec<Experience>,
    buttons: Vec<(web_sys::HtmlElement, ExperienceFilter)>,
    filter: ExperienceFilter,
    rx: mpsc::Receiver<ExperienceFilter>,
}

impl ExperienceController {
    pub async fn new() -> Self {
        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let experiences_list = document
            .get_element_by_id("experiences-list")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let response = JsFuture::from(window.fetch_with_str("./assets/experiences.json"))
            .await
            .unwrap_throw()
            .dyn_into::<web_sys::Response>()
            .unwrap_throw();
        let json = JsFuture::from(response.json().unwrap_throw())
            .await
            .unwrap_throw();
        let experiences =
            serde_wasm_bindgen::from_value::<Vec<ExperienceData>>(json).unwrap_throw();

        let html = Self::render_experiences_list(&experiences);

        experiences_list.set_inner_html(&html);

        let experiences = experiences
            .into_iter()
            .enumerate()
            .map(|(i, data)| {
                let element = document
                    .get_element_by_id(&format!("experience-{}", i))
                    .unwrap_throw()
                    .dyn_into::<web_sys::HtmlElement>()
                    .unwrap_throw();
                let height = element.get_bounding_client_rect().height() as f32;

                Experience {
                    data,
                    element,
                    height,
                }
            })
            .collect::<Vec<_>>();

        let (tx, rx) = mpsc::channel();
        let buttons = document
            .get_elements_by_class_name("experiences-filter-button")
            .iter()
            .map(|el| el.dyn_into::<web_sys::HtmlElement>().unwrap_throw())
            .map(|button| {
                let id = button.id();
                let filter = match id.as_str() {
                    "experiences-education-filter" => ExperienceFilter::Education,
                    "experiences-work-filter" => ExperienceFilter::Work,
                    "experiences-others-filter" => ExperienceFilter::Others,
                    _ => ExperienceFilter::Education,
                };

                add_event_listener!(button, "click", {
                    let tx = tx.clone();
                    move |_: web_sys::Event| {
                        if let Err(e) = tx.send(filter) {
                            log::error!("Failed to send experience filter: {e}");
                        }
                    }
                }; FnMut(_));

                (button, filter)
            })
            .collect();

        let this = Self {
            experiences,
            buttons,
            filter: ExperienceFilter::default(),
            rx,
        };

        this.update_buttons();
        this.update_elements();
        this
    }

    pub fn resize(&mut self) {
        for exp in &mut self.experiences {
            if let Err(e) = exp.element.style().set_property("height", "auto") {
                log::error!(
                    "Failed to reset experience height: {}",
                    e.as_string().unwrap_or("Unknown error".to_string())
                );
                continue;
            }

            exp.height = exp.element.get_bounding_client_rect().height() as f32;
        }

        self.update_elements();
    }

    pub fn update(&mut self) {
        if let Some(new_filter) = self.rx.try_recv().into_iter().next_back()
            && new_filter != self.filter
        {
            self.filter = new_filter;
            self.update_buttons();
            self.update_elements();
        }
    }

    fn update_buttons(&self) {
        for (button, filter) in &self.buttons {
            let bottom = if *filter == self.filter {
                "-52px"
            } else {
                "0px"
            };

            if let Err(e) = button.style().set_property("bottom", bottom) {
                log::error!(
                    "Failed to update experience filter button position: {}",
                    e.as_string().unwrap_or("Unknown error".to_string())
                );
            }
        }
    }

    fn update_elements(&self) {
        for (i, exp) in self.experiences.iter().enumerate() {
            let (height, opacity) = if exp.data.filter == self.filter {
                (format!("{}px", exp.height), "1")
            } else {
                ("0px".to_string(), "0")
            };

            let height_result = exp.element.style().set_property("height", &height);
            let opacity_result = exp.element.style().set_property("opacity", opacity);

            if let Err(e) = height_result.or(opacity_result) {
                log::error!(
                    "Failed to update experience {i} display: {}",
                    e.as_string().unwrap_or("Unknown error".to_string())
                );
            }
        }
    }

    fn render_experiences_list(experiences: &[ExperienceData]) -> String {
        let experiences_html = Self::render_experiences(experiences);

        format!(
            "
            <div class=\"panel sized-panel\" style=\"padding: 12px 24px\">
                {}
            </div>
            ",
            experiences_html
        )
    }

    fn render_experiences(experiences: &[ExperienceData]) -> String {
        experiences
            .iter()
            .enumerate()
            .map(|(i, experience)| {
                let start_date = format!(
                    "{} {}",
                    show_month(experience.start_month),
                    experience.start_year
                );
                let end_date = match (experience.end_year, experience.end_month) {
                    (Some(year), Some(month)) => format!("{} {}", show_month(month), year),
                    _ => "Present".to_string(),
                };
                format!(
                    "
                    <div id=\"experience-{}\" class=\"experience\">
                        <div>
                            <p>{} - {}</p>
                            <h3>{}</h3>
                            <p>{}</p>
                        </div>
                    </div>
                    ",
                    i, start_date, end_date, experience.name, experience.organization
                )
            })
            .collect::<String>()
    }
}

fn show_month(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "Unknown",
    }
}
