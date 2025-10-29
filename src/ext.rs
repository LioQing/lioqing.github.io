use glam::*;
use wasm_bindgen::UnwrapThrowExt as _;

pub trait Vec4Ext {
    fn to_wgpu_color(&self) -> wgpu::Color;
}

impl Vec4Ext for Vec4 {
    fn to_wgpu_color(&self) -> wgpu::Color {
        wgpu::Color {
            r: self.x as f64,
            g: self.y as f64,
            b: self.z as f64,
            a: self.w as f64,
        }
    }
}

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

    fn size(&self) -> IVec2;
}

impl WindowExt for web_sys::Window {
    fn scroll_pos(&self) -> IVec2 {
        IVec2::new(
            self.scroll_x().expect_throw("scroll x") as i32,
            self.scroll_y().expect_throw("scroll y") as i32,
        )
    }

    fn size(&self) -> IVec2 {
        let doc_elem = self
            .document()
            .expect_throw("document")
            .document_element()
            .expect_throw("document element");
        IVec2::new(doc_elem.client_width(), doc_elem.client_height())
    }
}

pub trait HtmlCollectionExt {
    fn iter(&self) -> impl Iterator<Item = web_sys::Element>;
}

impl HtmlCollectionExt for web_sys::HtmlCollection {
    fn iter(&self) -> impl Iterator<Item = web_sys::Element> {
        (0..self.length()).filter_map(|i| self.item(i))
    }
}

pub trait DomRectExt {
    fn top_left(&self) -> Vec2;

    fn size(&self) -> Vec2;
}

impl DomRectExt for web_sys::DomRect {
    fn top_left(&self) -> Vec2 {
        Vec2::new(self.left() as f32, self.top() as f32)
    }

    fn size(&self) -> Vec2 {
        Vec2::new(self.width() as f32, self.height() as f32)
    }
}

pub trait MouseEventExt {
    fn client_position(&self) -> IVec2;
}

impl MouseEventExt for web_sys::MouseEvent {
    fn client_position(&self) -> IVec2 {
        IVec2::new(self.client_x(), self.client_y())
    }
}
