use std::{cell::OnceCell, rc::Rc, sync::mpsc};

use glam::*;
use wasm_bindgen::prelude::*;

use crate::{
    background::{Background, BackgroundEvent},
    ext::CanvasExt as _,
    gpu::Gpu,
};

mod background;
mod ext;
mod frame;
mod gpu;
mod images;
mod line_segment;
mod logger;
mod meta_field;
mod meta_image;
mod meta_shape;
mod mouse;
mod pipeline;

#[macro_export]
macro_rules! add_event_listener {
    ($target:expr, $event:expr, $closure:expr, as $($trait:tt)+) => {
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

    let canvas = document
        .get_element_by_id("background")
        .unwrap_throw()
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap_throw();

    wasm_bindgen_futures::spawn_local(async move {
        let gpu = Gpu::new(canvas.clone()).await;

        let (tx, rx) = mpsc::channel();
        let mut background = Background::new(gpu, canvas.clone(), rx);
        add_event_listener!(window, "mousemove", {
            let tx = tx.clone();
            move |event: web_sys::MouseEvent| {
                let pos = ivec2(
                    event.client_x(),
                    event.client_y(),
                );
                if let Err(e) = tx.send(BackgroundEvent::MouseMove(pos)) {
                    log::error!("Failed to send mouse move event: {e}");
                }
            }
        }, as FnMut(_));
        add_event_listener!(window, "resize", {
            let tx = tx.clone();
            let canvas = canvas.clone();
            move || {
                let size = canvas.size();
                if let Err(e) = tx.send(BackgroundEvent::Resize(size)) {
                    log::error!("Failed to send resize event: {e}");
                }
            }
        }, as FnMut());

        log::debug!("Background initialized");

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
