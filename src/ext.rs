use glam::*;
use wasm_bindgen::UnwrapThrowExt as _;

pub trait CanvasExt {
    fn size(&self) -> UVec2;
}

impl CanvasExt for web_sys::HtmlCanvasElement {
    fn size(&self) -> UVec2 {
        UVec2::new(
            self.client_width().max(0) as u32,
            self.client_height().max(0) as u32,
        )
    }
}

pub trait SurfaceConfigurationExt {
    fn size(&self) -> UVec2;

    fn is_valid(&self) -> bool {
        self.size().x > 0 && self.size().y > 0
    }
}

impl SurfaceConfigurationExt for wgpu::SurfaceConfiguration {
    fn size(&self) -> UVec2 {
        UVec2::new(self.width, self.height)
    }
}

pub trait WindowExt {
    fn scroll_pos(&self) -> IVec2;
}

impl WindowExt for web_sys::Window {
    fn scroll_pos(&self) -> IVec2 {
        IVec2::new(
            self.scroll_x().unwrap_throw() as i32,
            self.scroll_y().unwrap_throw() as i32,
        )
    }
}
