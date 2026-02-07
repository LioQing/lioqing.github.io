use std::{cell::OnceCell, rc::Rc, sync::mpsc};

use wasm_bindgen::prelude::*;

use crate::{
    background::{Background, BackgroundEvent},
    ext::MouseEventExt as _,
    gpu::Gpu,
    theme::Theme,
};

mod background;
mod controller;
mod delta_time;
mod event_listeners;
mod ext;
mod frame;
mod gpu;
mod grid;
mod logger;
mod mar_sq;
mod meta_field;
mod meta_shape;
mod mouse;
mod pipeline;
mod texture_blitter;
mod theme;

#[macro_export]
macro_rules! add_event_listener {
    ($target:expr, $event:expr, $closure:expr; $($trait:tt)+) => {
        let closure = wasm_bindgen::prelude::Closure::wrap(Box::new($closure) as Box<dyn $($trait)+>);
        $target
            .add_event_listener_with_callback($event, closure.as_ref().unchecked_ref())
            .unwrap_throw();
        closure.forget();
    };
}

fn main() {
    logger::init(if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    });

    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap_throw();

    let document = window.document().unwrap_throw();

    let search = window.location().search().unwrap_throw();
    let params = web_sys::UrlSearchParams::new_with_str(&search).unwrap_throw();
    let bgvfx_enabled = params.get("bgvfx") == Some("1".to_string());

    Theme::set_current(Theme::Dark);

    wasm_bindgen_futures::spawn_local(async move {
        event_listeners::init().await;

        if !bgvfx_enabled {
            log::debug!("Background VFX disabled");
            document
                .get_element_by_id("loading-cover")
                .unwrap_throw()
                .set_attribute("style", "display: none;")
                .unwrap_throw();
            return;
        }

        let canvas = document
            .get_element_by_id("background")
            .unwrap_throw()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap_throw();

        let gpu = Gpu::new(canvas.clone()).await;

        let (tx, rx) = mpsc::channel();
        let mut background = Background::new(gpu, canvas.clone(), rx).await;
        add_event_listener!(window, "pointermove", {
            let tx = tx.clone();
            move |event: web_sys::PointerEvent| {
                if event.pointer_type() != "mouse" {
                    return;
                }

                if let Err(e) = tx.send(BackgroundEvent::MouseMove(event.client_position())) {
                    log::error!("Failed to send mouse move event: {e}");
                }
            }
        }; FnMut(_));
        add_event_listener!(window.visual_viewport().unwrap_throw(), "resize", {
            let tx = tx.clone();
            move || {
                if let Err(e) = tx.send(BackgroundEvent::Resize) {
                    log::error!("Failed to send resize event: {e}");
                }
            }
        }; FnMut());

        log::debug!("Background initialized");

        event_listeners::cleanup_doc_for_bgvfx().await;

        document
            .get_element_by_id("loading-cover")
            .unwrap_throw()
            .set_attribute("style", "display: none;")
            .unwrap_throw();

        let update = Rc::<OnceCell<Closure<dyn FnMut()>>>::default();
        update
            .set(Closure::wrap(Box::new({
                let update = update.clone();
                let window = window.clone();
                move || {
                    background.update();
                    window
                        .request_animation_frame(update.get().unwrap().as_ref().unchecked_ref())
                        .unwrap_throw();
                }
            }) as Box<dyn FnMut()>))
            .unwrap_throw();
        window
            .request_animation_frame(update.get().unwrap_throw().as_ref().unchecked_ref())
            .unwrap_throw();
    });
}
