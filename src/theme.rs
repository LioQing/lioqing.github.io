use std::sync::{LazyLock, atomic::Ordering};

use ahash::HashMap;
use glam::*;
use wasm_bindgen::{JsCast as _, UnwrapThrowExt as _};

static CURRENT_THEME: AtomicTheme = AtomicTheme::new(Theme::Dark);

#[atomic_enum::atomic_enum]
#[derive(PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn properties(&self) -> &'static HashMap<ThemePropertyName, ThemeProperty> {
        match self {
            Theme::Dark => dark(),
            Theme::Light => unimplemented!(),
        }
    }

    pub fn current() -> Self {
        CURRENT_THEME.load(Ordering::Relaxed)
    }

    pub fn set_current(theme: Theme) {
        CURRENT_THEME.store(theme, Ordering::Relaxed);

        log::debug!("Switched to {theme:?}");

        let style = web_sys::window()
            .expect_throw("window")
            .document()
            .expect_throw("document")
            .document_element()
            .expect_throw("document element")
            .dyn_into::<web_sys::HtmlElement>()
            .expect_throw("html element")
            .style();

        for (name, property) in theme.properties() {
            style
                .set_property(&format!("--{name}"), &property.value())
                .expect_throw("set property");
        }
    }
}

#[derive(Debug, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Hash, strum::Display))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
#[strum_discriminants(name(ThemePropertyName))]
pub enum ThemeProperty {
    Background(Vec4),
    Foreground(Vec4),
    Ttd(Vec4),
    Hku(Vec4),
    Hk(Vec4),
}

impl ThemeProperty {
    pub fn value(&self) -> String {
        match self {
            ThemeProperty::Background(color)
            | ThemeProperty::Foreground(color)
            | ThemeProperty::Ttd(color)
            | ThemeProperty::Hku(color)
            | ThemeProperty::Hk(color) => {
                let r = (color.x * 255.0).round() as u8;
                let g = (color.y * 255.0).round() as u8;
                let b = (color.z * 255.0).round() as u8;
                let a = color.w;
                format!("rgba({r}, {g}, {b}, {a})")
            }
        }
    }

    pub fn vec4(&self) -> Option<Vec4> {
        match self {
            ThemeProperty::Background(color)
            | ThemeProperty::Foreground(color)
            | ThemeProperty::Ttd(color)
            | ThemeProperty::Hku(color)
            | ThemeProperty::Hk(color) => Some(*color),
        }
    }
}

fn dark() -> &'static HashMap<ThemePropertyName, ThemeProperty> {
    static THEME: LazyLock<HashMap<ThemePropertyName, ThemeProperty>> = LazyLock::new(|| {
        [
            (
                ThemePropertyName::Background,
                ThemeProperty::Background(vec4(0.11, 0.11, 0.11, 1.0)),
            ),
            (
                ThemePropertyName::Foreground,
                ThemeProperty::Foreground(vec4(0.87, 0.87, 0.87, 1.0)),
            ),
            (
                ThemePropertyName::Ttd,
                ThemeProperty::Ttd(vec4(3.0 / 256.0, 114.0 / 256.0, 226.0 / 256.0, 1.0)),
            ),
            (
                ThemePropertyName::Hku,
                ThemeProperty::Hku(vec4(78.0 / 256.0, 189.0 / 256.0, 136.0 / 256.0, 1.0)),
            ),
            (
                ThemePropertyName::Hk,
                ThemeProperty::Hk(vec4(238.0 / 256.0, 28.0 / 256.0, 37.0 / 256.0, 1.0)),
            ),
        ]
        .into_iter()
        .collect()
    });

    &THEME
}
