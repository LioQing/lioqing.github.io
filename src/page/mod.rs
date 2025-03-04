use leptos::prelude::*;
use leptos_router::hooks::use_location;

use crate::consts::{FRAME_WIDTH, SMALL_SPACER_HEIGHT, SPACER_HEIGHT};

mod experiences;
mod home;
mod not_found;
mod projects;
mod skills;

pub use experiences::*;
pub use home::*;
pub use not_found::*;
pub use projects::*;
pub use skills::*;

#[component]
pub fn Routes() -> impl IntoView {
    let location = use_location();

    move || match location.hash.get().to_lowercase().as_str() {
        "" => view! { <Home /> }.into_any(),
        "#skills" => view! { <Skills /> }.into_any(),
        "#projects" => view! { <Projects /> }.into_any(),
        "#experiences" => view! { <Experiences /> }.into_any(),
        _ => view! { <NotFound /> }.into_any(),
    }
}

#[component(transparent)]
pub fn Spacer(#[prop(default = false)] small: bool) -> impl IntoView {
    view! {
        <div style=format!("height: {};", if small { SMALL_SPACER_HEIGHT } else { SPACER_HEIGHT })/>
    }
}

#[component]
pub fn Content(
    icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div style=format!("\
            display: flex; \
            flex-direction: column; \
            justify-content: start; \
            align-items: start; \
            min-height: calc(100% - {FRAME_WIDTH} * 2); \
            padding: 2rem; \
        ")>
            <div style="\
                display: flex; \
                flex-direction: row; \
                justify-content: start; \
                align-items: center; \
                flex-flow: row wrap; \
                gap: 1em; \
                padding-bottom: 0.4em; \
            ">
                <i class=format!("ph {icon}") style="font-size: 3em;"/>
                <h2>{title}.</h2>
            </div>
            <p>{subtitle}</p>
            {children()}
        </div>
    }
}
