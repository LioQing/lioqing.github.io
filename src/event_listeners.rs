use glam::*;
use wasm_bindgen::{JsCast as _, UnwrapThrowExt as _};
use wasm_bindgen_futures::JsFuture;
use web_time::web;

use crate::{
    add_event_listener,
    ext::{DomRectExt as _, HtmlCollectionExt as _, MouseEventExt},
};

pub fn init() {
    let window = web_sys::window().unwrap_throw();

    let document = window.document().unwrap_throw();

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
        let scroll_down_indicator_element = document
            .get_element_by_id("scroll-down-indicator")
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
                if let Err(e) = scroll_down_indicator_element.style().set_property(
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

    // Experiences filter
    wasm_bindgen_futures::spawn_local(async move {
        #[derive(Debug, Clone, Copy, strum::Display, serde::Deserialize)]
        #[strum(serialize_all = "lowercase")]
        #[serde(rename_all = "lowercase")]
        enum Filter {
            Education,
            Work,
            Others,
        }

        #[derive(Debug, Clone, serde::Deserialize)]
        struct Experience {
            name: String,
            organization: String,
            filter: Filter,
            start_year: u32,
            start_month: u32,
            end_year: Option<u32>,
            end_month: Option<u32>,
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

        let experiences_list = document
            .get_element_by_id("experiences-list")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap_throw();

        let response = JsFuture::from(window.fetch_with_str("assets/experiences.json"))
            .await
            .unwrap_throw()
            .dyn_into::<web_sys::Response>()
            .unwrap_throw();
        let json = JsFuture::from(response.json().unwrap_throw())
            .await
            .unwrap_throw();
        let experiences = serde_wasm_bindgen::from_value::<Vec<Experience>>(json).unwrap_throw();

        let experiences_html = experiences
            .into_iter()
            .map(|experience| {
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
                    <div class=\"experience\">
                        <div>
                            <h3>{}</h3>
                            <p>{}</p>
                            <p>{} - {}</p>
                        </div>
                    </div>
                    ",
                    experience.name, experience.organization, start_date, end_date
                )
            })
            .collect::<String>();

        let html =
            format!("<div class=\"panel\" style=\"padding: 12px 24px\">{experiences_html}</div>",);

        experiences_list.set_inner_html(&html);
    });
}
