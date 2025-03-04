#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct AppState {
    pub theme: Theme,
    pub background_simulated: BackgroundSimulated,
}

#[cfg(target_arch = "wasm32")]
pub mod hooks {
    use codee::string::JsonSerdeCodec;
    use leptos::{prelude::*, reactive::wrappers::write::SignalSetter};
    use leptos_use::storage::use_local_storage;

    use super::*;

    pub fn use_app_state() -> (Signal<AppState>, WriteSignal<AppState>) {
        let (app_state, set_app_state, _) =
            use_local_storage::<AppState, JsonSerdeCodec>("lioqing_app_state");
        (app_state, set_app_state)
    }

    pub fn use_theme() -> (Signal<Theme>, SignalSetter<Theme>) {
        let (app_state, set_app_state) = use_app_state();
        let get = Signal::derive(move || app_state.get().theme);
        let set = SignalSetter::map(move |theme| set_app_state.update(|state| state.theme = theme));
        (get, set)
    }

    pub fn use_background_simulated() -> (
        Signal<BackgroundSimulated>,
        SignalSetter<BackgroundSimulated>,
    ) {
        let (app_state, set_app_state) = use_app_state();
        let get = Signal::derive(move || app_state.get().background_simulated);
        let set = SignalSetter::map(move |background_simulated| {
            set_app_state.update(|state| state.background_simulated = background_simulated)
        });
        (get, set)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Theme {
    Light,

    #[default]
    Dark,
}

impl Theme {
    pub const fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "Light",
            Theme::Dark => "Dark",
        }
    }

    pub const fn toggle(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }

    pub const fn background_color(self) -> &'static str {
        match self {
            Theme::Light => "#f0f0f0",
            Theme::Dark => "#151515",
        }
    }

    pub const fn background_color_hex(self) -> u32 {
        match self {
            Theme::Light => 0xf0f0f0,
            Theme::Dark => 0x151515,
        }
    }

    pub const fn foreground_color(self) -> &'static str {
        self.toggle().background_color()
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct BackgroundSimulated(bool);

impl BackgroundSimulated {
    pub const fn new(simulated: bool) -> Self {
        Self(simulated)
    }

    pub const fn toggle(self) -> Self {
        Self::new(!self.0)
    }

    pub const fn is_simulated(self) -> bool {
        self.0
    }
}

impl From<BackgroundSimulated> for bool {
    fn from(background_simulated: BackgroundSimulated) -> Self {
        background_simulated.0
    }
}

impl From<bool> for BackgroundSimulated {
    fn from(simulated: bool) -> Self {
        Self::new(simulated)
    }
}
