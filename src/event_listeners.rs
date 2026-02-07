use glam::*;
use wasm_bindgen::{JsCast as _, UnwrapThrowExt as _};
use wasm_bindgen_futures::JsFuture;

use crate::{add_event_listener, ext::HtmlCollectionExt as _};

pub async fn init() {
    let window = web_sys::window().unwrap_throw();

    let document = window.document().unwrap_throw();

    // BG VFX link
    {
        let bgvfx_link = document
            .get_element_by_id("bgvfx-link")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();
        let window = window.clone();

        add_event_listener!(bgvfx_link, "click", {
            let window = window.clone();
            move |_event: web_sys::Event| {
                let Ok(location) = window.location().href() else {
                    log::warn!("Failed to read current location for bgvfx");
                    return;
                };
                let Ok(url) = web_sys::Url::new(&location) else {
                    log::warn!("Failed to parse location for bgvfx");
                    return;
                };
                url.search_params().set("bgvfx", "1");
                if let Err(e) = window.location().set_href(&url.to_string().as_string().unwrap_throw()) {
                    log::warn!("Failed to set bgvfx location: {e:?}");
                }
            }
        }; FnMut(_));
    }

    // Parallax
    {
        let first_name_element = document
            .get_element_by_id("first-name")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();
        let last_name_element = document
            .get_element_by_id("last-name")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();
        let software_title_element = document
            .get_element_by_id("software-title")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();
        let engineer_title_element = document
            .get_element_by_id("engineer-title")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();
        let title_overlay_element = document
            .get_element_by_id("title-overlay")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();
        add_event_listener!(window, "scroll", {
            let window = window.clone();
            move |_: web_sys::Event| {
                let scroll_y = match window.scroll_y().map(|v| v as f32) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!("Failed to get scroll y: {e:?}");
                        return;
                    }
                };
                
                let parallax_y = |element: &web_sys::HtmlElement, exponent: f32| {
                    if let Err(e) = element.style().set_property(
                        "transform",
                        &format!("translateY({}px)", -scroll_y.max(0.0).powf(exponent) / 360.0),
                    ) {
                        log::warn!("Failed to set transform: {e:?}");
                    }
                };

                parallax_y(&first_name_element, 1.95);
                parallax_y(&last_name_element, 1.85);
                parallax_y(&software_title_element, 1.6);
                parallax_y(&engineer_title_element, 1.8);

                let indicator_opacity = (1.0 - scroll_y / 150.0).clamp(0.0, 1.0);
                if let Err(e) = title_overlay_element.style().set_property(
                    "opacity",
                    &indicator_opacity.to_string(),
                ) {
                    log::warn!("Failed to set opacity: {e:?}");
                }
            }
        }; FnMut(_));
    }

    // About me logo heights
    {
        fn parse_px(value: &str) -> Option<f64> {
            let trimmed = value.trim();
            if trimmed.is_empty()
                || trimmed.eq_ignore_ascii_case("auto")
                || trimmed.eq_ignore_ascii_case("normal")
            {
                return None;
            }
            trimmed
                .strip_suffix("px")
                .or_else(|| trimmed.strip_suffix("PX"))
                .map(str::trim)
                .and_then(|number| number.parse::<f64>().ok())
        }

        let update_about_me_logo_heights = || {
            let window = web_sys::window().expect_throw("window");
            let document = window.document().expect_throw("document");
            for element in document
                .get_elements_by_class_name("about-me-profile-logo")
                .iter()
            {
                let logo = match element.dyn_into::<web_sys::HtmlElement>() {
                    Ok(logo) => logo,
                    Err(e) => {
                        log::warn!("About me profile logo element is not an HtmlElement: {e:?}");
                        continue;
                    }
                };
                let Some(parent_element) = logo.parent_element() else {
                    log::warn!("Logo element has no parent");
                    continue;
                };
                let parent = match parent_element.dyn_into::<web_sys::HtmlElement>() {
                    Ok(parent) => parent,
                    Err(e) => {
                        log::warn!("Parent element is not an HtmlElement: {e:?}");
                        continue;
                    }
                };

                let parent_height = f64::from(parent.client_height());

                let computed_style = match window.get_computed_style(&parent) {
                    Ok(Some(style)) => style,
                    Ok(None) => {
                        log::warn!("Failed to get computed style for parent element");
                        continue;
                    }
                    Err(e) => {
                        log::warn!("Failed to get computed style: {e:?}");
                        continue;
                    }
                };

                let padding_top = computed_style
                    .get_property_value("padding-top")
                    .ok()
                    .and_then(|value| parse_px(&value))
                    .unwrap_or(0.0);
                let padding_bottom = computed_style
                    .get_property_value("padding-bottom")
                    .ok()
                    .and_then(|value| parse_px(&value))
                    .unwrap_or(0.0);
                let padding = computed_style
                    .get_property_value("padding")
                    .ok()
                    .and_then(|value| parse_px(&value))
                    .unwrap_or(0.0);
                let gap_per_row = computed_style
                    .get_property_value("row-gap")
                    .ok()
                    .or_else(|| computed_style.get_property_value("gap").ok())
                    .and_then(|value| parse_px(&value))
                    .unwrap_or(0.0);

                let children = parent.children();
                let mut siblings_height = 0.0;
                let logo_element = logo
                    .clone()
                    .dyn_into::<web_sys::Element>()
                    .expect_throw("logo element");
                for child_index in 0..children.length() {
                    let Some(child) = children.item(child_index) else {
                        log::warn!("Failed to get child element at index {child_index}");
                        continue;
                    };
                    if child == logo_element {
                        continue;
                    }
                    let child_element = match child.dyn_into::<web_sys::HtmlElement>() {
                        Ok(elem) => elem,
                        Err(e) => {
                            log::warn!("Child element is not an HtmlElement: {e:?}");
                            continue;
                        }
                    };
                    siblings_height += f64::from(child_element.offset_height());
                }

                let gap_total = gap_per_row * f64::from(children.length().saturating_sub(1));
                let available_height = (parent_height
                    - padding_top
                    - padding_bottom
                    - padding
                    - siblings_height
                    - gap_total)
                    .max(0.0);

                if let Err(e) = logo
                    .style()
                    .set_property("height", &format!("{available_height}px"))
                {
                    log::warn!("Failed to set about-me-profile-logo height: {e:?}");
                }
            }
        };

        update_about_me_logo_heights();
        add_event_listener!(window, "resize", move || {
            update_about_me_logo_heights();
        }; FnMut());
    }

    // Tilting
    // {
    //     const MAX_TILT_ANGLE: f32 = 10.0;

    //     let tilting_containers = document.get_elements_by_class_name("tilting-container");
    //     for tilting_container in tilting_containers.iter() {
    //         let tilting_container = tilting_container;

    //         let tilting = tilting_container
    //             .get_elements_by_class_name("tilting")
    //             .iter()
    //             .next()
    //             .unwrap_throw()
    //             .dyn_into::<web_sys::HtmlElement>()
    //             .unwrap_throw();

    //         add_event_listener!(tilting_container, "mousemove", {
    //             let tilting_container = tilting_container.clone();
    //             let tilting = tilting.clone();
    //             move |event: web_sys::MouseEvent| {
    //                 let rect = tilting_container.get_bounding_client_rect();
    //                 let mouse = (event.client_position().as_vec2() - rect.top_left()) / rect.size();
    //                 let dir = (mouse - Vec2::splat(0.5)) * Vec2::new(1.0, -1.0) * 2.0;
    //                 let rotate = match dir.length_squared() {
    //                     d if d >= 1.0 => Vec2::ZERO,
    //                     d => dir.normalize() * (d * std::f32::consts::PI).sin() * MAX_TILT_ANGLE,
    //                 };
    //                 if let Err(e) = tilting.style().set_property(
    //                     "transform",
    //                     &format!(
    //                         "rotateX({}deg) rotateY({}deg)",
    //                         rotate.y, rotate.x
    //                     ),
    //                 ) {
    //                     log::warn!("Failed to set tilting transform: {e:?}");
    //                 }
    //             }
    //         }; FnMut(_));

    //         add_event_listener!(tilting_container, "mouseleave", {
    //             move || {
    //                 if let Err(e) = tilting.style().set_property("transform", "rotateX(0deg) rotateY(0deg)") {
    //                     log::warn!("Failed to reset tilting transform: {e:?}");
    //                 }
    //             }
    //         }; FnMut());
    //     }
    // }

    // Projects
    {
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

        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let element = document
            .get_element_by_id("projects")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let response = JsFuture::from(window.fetch_with_str("projects.json"))
            .await
            .unwrap_throw()
            .dyn_into::<web_sys::Response>()
            .unwrap_throw();
        let json = JsFuture::from(response.json().unwrap_throw())
            .await
            .unwrap_throw();
        let projects = serde_wasm_bindgen::from_value::<Vec<ProjectData>>(json).unwrap_throw();

        let html = render_project_list(&projects);
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
                                set_project_details_expanded(
                                    detail,
                                    !is_expanded,
                                );
                            } else {
                                set_project_details_expanded(detail, false);
                            }
                        }
                    }
                }; FnMut(_));
        }

        fn render_project_list(projects: &[ProjectData]) -> String {
            let items = projects
                .iter()
                .enumerate()
                .map(|(i, project)| render_project_list_item(i, project))
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
            let tags_html = render_project_tags(&project.tags);
            let date = format!("{} {}", show_month(project.month), project.year);
            let details_html = render_project_details(project);

            format!(
                "
                <div
                    class=\"panel interactive-panel project-item-panel\"
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
            let links_html = render_project_links(&project.links);

            format!(
                "
                <div style=\"
                    padding: 0px 36px 36px 36px;
                    display: flex;
                    flex-direction: row;
                    gap: 24px;
                    flex-wrap: wrap;
                \">
                    <img src=\"projects/{}\" alt=\"{}\" style=\"
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

    // Experiences
    {
        #[derive(
            Debug, Default, Clone, Copy, PartialEq, Eq, strum::Display, serde::Deserialize,
        )]
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

        #[derive(Debug, Clone)]
        struct Experience {
            data: ExperienceData,
            element: web_sys::HtmlElement,
        }

        let window = web_sys::window().expect_throw("window");
        let document = window.document().expect_throw("document");

        let experiences_list = document
            .get_element_by_id("experiences-list")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let response = JsFuture::from(window.fetch_with_str("experiences.json"))
            .await
            .unwrap_throw()
            .dyn_into::<web_sys::Response>()
            .unwrap_throw();
        let json = JsFuture::from(response.json().unwrap_throw())
            .await
            .unwrap_throw();
        let experiences =
            serde_wasm_bindgen::from_value::<Vec<ExperienceData>>(json).unwrap_throw();

        let html = render_experiences_list(&experiences);

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

                Experience { data, element }
            })
            .collect::<Vec<_>>();

        let mut buttons = document
            .get_elements_by_class_name("experiences-filter-button")
            .iter()
            .map(|el| el.dyn_into::<web_sys::HtmlElement>().unwrap_throw())
            .map(|button| {
                let id = button.id();
                let filter = match id.as_str() {
                    "experiences-work-filter" => ExperienceFilter::Work,
                    "experiences-education-filter" => ExperienceFilter::Education,
                    "experiences-others-filter" => ExperienceFilter::Others,
                    _ => ExperienceFilter::Education,
                };

                (button, filter)
            })
            .collect::<Vec<_>>();

        for (button, filter) in &buttons {
            let filter = *filter;

            add_event_listener!(button, "click", {
                let mut buttons = buttons.clone();
                let experiences = experiences.clone();
                move |_: web_sys::Event| {
                    update_buttons(&mut buttons, filter);
                    update_elements(&experiences, filter);
                }
            }; FnMut(_));

            add_event_listener!(button, "resize", {
                let experiences = experiences.clone();
                move |_: web_sys::Event| {
                    update_elements(&experiences, filter);
                }
            }; FnMut(_));
        }

        update_buttons(&mut buttons, ExperienceFilter::default());
        update_elements(&experiences, ExperienceFilter::default());

        fn update_buttons(
            buttons: &mut [(web_sys::HtmlElement, ExperienceFilter)],
            filter: ExperienceFilter,
        ) {
            for (button, button_filter) in buttons {
                let bottom = if *button_filter == filter {
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

        fn update_elements(experiences: &[Experience], filter: ExperienceFilter) {
            for (i, exp) in experiences.iter().enumerate() {
                let (height, opacity) = if exp.data.filter == filter {
                    ("auto", "1")
                } else {
                    ("0px", "0")
                };

                let height_result = exp.element.style().set_property("height", height);
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
            let experiences_html = render_experiences(experiences);

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
}

pub async fn cleanup_doc_for_bgvfx() {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    if let Some(element) = document.get_element_by_id("bgvfx-link") {
        element.remove();
    }

    if let Some(element) = document.get_element_by_id("background-image") {
        element.remove();
    }

    if let Some(element) = document.get_element_by_id("skill-icons") {
        element.remove();
    }

    let style_sheets = document.style_sheets();

    for sheet_index in 0..style_sheets.length() {
        let Some(sheet) = style_sheets.item(sheet_index) else {
            continue;
        };

        let css_sheet = match sheet.dyn_into::<web_sys::CssStyleSheet>() {
            Ok(sheet) => sheet,
            Err(_) => continue,
        };

        let rules = match css_sheet.css_rules() {
            Ok(rules) => rules,
            Err(e) => {
                log::warn!("Failed to read css rules: {e:?}");
                continue;
            }
        };

        for rule_index in (0..rules.length()).rev() {
            let Some(rule) = rules.item(rule_index) else {
                continue;
            };

            let style_rule = match rule.dyn_into::<web_sys::CssStyleRule>() {
                Ok(rule) => rule,
                Err(_) => continue,
            };

            let selector = style_rule.selector_text();
            let should_remove = matches!(
                selector.as_str(),
                ".interactive-panel:hover"
                    | ".interactive-panel.project-item-panel:hover"
                    | ".panel::before"
                    | ".panel"
            );

            if should_remove && let Err(e) = css_sheet.delete_rule(rule_index) {
                log::warn!("Failed to delete css rule {selector}: {e:?}");
            }
        }
    }

    let style = document
        .create_element("style")
        .expect_throw("style element");
    style.set_text_content(Some(".panel { border-radius: 36px; }"));
    if let Some(head) = document.head() {
        if let Err(e) = head.append_child(&style) {
            log::warn!("Failed to append cleanup style: {e:?}");
        }
    } else {
        log::warn!("Failed to locate document head for cleanup style");
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
