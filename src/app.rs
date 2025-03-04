use std::sync::mpsc::Sender;

use chrono::Datelike;
use leptos::prelude::*;
use leptos_router::components::Router;
use leptos_router::hooks::use_location;
use leptos_use::{use_element_size, use_timestamp, UseElementSizeReturn};

use crate::background::{background, BackgroundCommand};
use crate::consts::{DEFAULT_TRANSITION, FRAME_BACKDROP_FILTER, FRAME_WIDTH};
use crate::page::Routes;
use crate::state::hooks::{use_background_simulated, use_theme};
use crate::state::Theme;

#[component]
pub fn App() -> impl IntoView {
    let (background_tx, set_background_tx) = signal::<Option<Sender<BackgroundCommand>>>(None);
    let (theme, _) = use_theme();
    let (background_simulated, _) = use_background_simulated();

    let background_ref = NodeRef::<leptos::html::Div>::new();
    let UseElementSizeReturn {
        width: background_width,
        height: background_height,
    } = use_element_size(background_ref);

    Effect::new(move || match background_simulated.get().is_simulated() {
        true => {
            if background_tx.get().is_some() {
                return;
            }

            let (tx, rx) = std::sync::mpsc::channel();

            wasm_bindgen_futures::spawn_local(async move {
                background(rx);
            });

            tx.send(BackgroundCommand::CreateCanvas("background"))
                .expect("send create canvas");

            set_background_tx.set(Some(tx));
        }
        false => {
            if let Some(tx) = background_tx.get() {
                tx.send(BackgroundCommand::Destroy).expect("send destroy");
                set_background_tx.set(None);
            }
        }
    });

    Effect::new(move || {
        background_tx.with(|tx| {
            if let Some(tx) = tx {
                tx.send(BackgroundCommand::Resize(
                    background_width.get() as f32,
                    background_height.get() as f32,
                ))
                .expect("send set size");
            }
        });
    });

    Effect::new(move || {
        background_tx.with(|tx| {
            if let Some(tx) = tx {
                tx.send(BackgroundCommand::SetTheme(theme.get()))
                    .expect("send set theme");
            }
        });
    });

    view! {
        <div
            style=move || {
                let foreground_color = theme.get().foreground_color();
                format!("\
                    color: {foreground_color}; \
                    transition: {DEFAULT_TRANSITION}; \
                ")
            }
            on:pointermove=move |e| {
                background_tx.with(|tx| {
                    if let Some(tx) = tx {
                        tx.send(BackgroundCommand::PointerMove(
                            e.client_x() as f32,
                            e.client_y() as f32,
                        ))
                        .expect("send pointer move");
                    }
                });
            }
            on:pointerdown=move |e| {
                background_tx.with(|tx| {
                    if let Some(tx) = tx {
                        tx.send(BackgroundCommand::PointerDown(e.pressure()))
                            .expect("send pointer down");
                    }
                });
            }
        >
            <Router>
                <div
                    style=format!("\
                        overflow-y: auto; \
                        width: calc(100vw - {FRAME_WIDTH} * 2); \
                        height: calc(100vh - {FRAME_WIDTH} * 2); \
                        padding: {FRAME_WIDTH}; \
                    ")
                >
                    <Routes/>
                </div>
                <Frame/>
            </Router>
        </div>
        <div
            node_ref=background_ref
            id="background"
            style=move || {
                let display = match background_simulated.get().is_simulated() {
                    true => "block",
                    false => "none",
                };
                format!("\
                    position: fixed; \
                    top: 0; \
                    left: 0; \
                    width: 100vw; \
                    height: 100vh; \
                    z-index: -1; \
                    display: {display}; \
                ")
            }
        />
        <div style=move || {
            let background_color = theme.get().background_color();
            format!("\
                position: fixed; \
                top: 0; \
                left: 0; \
                width: 100vw; \
                height: 100vh; \
                z-index: -2; \
                background-color: {background_color}; \
                transition: {DEFAULT_TRANSITION}; \
            ")
        }>
            <img
                src="/assets/images/Logo.svg"
                alt="logo"
                style=format!("\
                    position: absolute; \
                    top: 50%; \
                    left: 75%; \
                    transform: translate(-50%, -50%); \
                    width: calc(50vw - {FRAME_WIDTH} * 2); \
                    height: calc(100vh - {FRAME_WIDTH} * 2); \
                    padding: {FRAME_WIDTH}; \
                    box-sizing: border-box; \
                ")
            />
        </div>
    }
}

#[component]
fn Frame() -> impl IntoView {
    let location = use_location();
    let timestamp = use_timestamp();
    let (theme, _) = use_theme();
    let (background_simulated, set_background_simulated) = use_background_simulated();

    view! {
        <div style=move || {
            let background_color = theme.get().background_color().to_string() + "80";
            format!("\
                position: fixed; \
                top: 0; \
                left: 0; \
                bottom: 0; \
                right: 0; \
                background-color: {background_color}; \
                backdrop-filter: {FRAME_BACKDROP_FILTER}; \
                mask-image: linear-gradient( \
                    to top, \
                    black 0% {FRAME_WIDTH}, \
                    transparent {FRAME_WIDTH} 100% \
                ), linear-gradient( \
                    to left, \
                    black 0% {FRAME_WIDTH}, \
                    transparent {FRAME_WIDTH} 100% \
                ), linear-gradient( \
                    to bottom, \
                    black 0% {FRAME_WIDTH}, \
                    transparent {FRAME_WIDTH} 100% \
                ), linear-gradient( \
                    to right, \
                    black 0% {FRAME_WIDTH}, \
                    transparent {FRAME_WIDTH} 100% \
                ); \
                pointer-events: none; \
            ")
        }/>
        <div style=format!("\
            position: fixed; \
            top: {FRAME_WIDTH}; \
            left: {FRAME_WIDTH}; \
            bottom: {FRAME_WIDTH}; \
            right: {FRAME_WIDTH}; \
            border: 1px solid; \
            pointer-events: none; \
        ")/>
        <Show when=move || location.hash.get() != "">
            <div style=format!("\
                position: fixed; \
                width: 0px; \
                height: 0px; \
                top: 1rem; \
                left: {FRAME_WIDTH}; \
                overflow: visible; \
            ")>
                <div style=format!("\
                    position: absolute; \
                    top: -0.6rem; \
                    width: calc(100vw - {FRAME_WIDTH} * 2); \
                    display: flex; \
                    justify-content: space-between; \
                    align-items: center; \
                ")>
                    <div style="display: flex; gap: 0.5rem; font-size: 0.8rem;">
                        <Navigation hash="skills" text="skills"/>
                        <Navigation hash="projects" text="projects"/>
                        <Navigation hash="experiences" text="experiences"/>
                    </div>
                    <div style="display: flex; gap: 0.2rem; font-size: 1rem;">
                        <a
                            href="https://github.com/LioQing"
                            target="_blank"
                            style="padding: 0.1rem;"
                        >
                            <i class="ph ph-github-logo"/>
                        </a>
                        <a
                            href="https://www.linkedin.com/in/lioqyz"
                            target="_blank"
                            style="padding: 0.1rem;"
                        >
                            <i class="ph ph-linkedin-logo"/>
                        </a>
                    </div>
                </div>
            </div>
        </Show>
        <div style=format!("\
            position: fixed; \
            width: 0px; \
            height: 0px; \
            left: 1rem; \
            bottom: {FRAME_WIDTH}; \
            transform: rotate(-90deg); \
            overflow: visible; \
        ")>
            <div style=format!("\
                position: absolute; \
                top: -0.6rem; \
                width: calc(100vh - {FRAME_WIDTH} * 2); \
                display: flex; \
                justify-content: space-between; \
                align-items: center; \
            ")>
                <div style="\
                    display: flex; \
                    justify-content: start; \
                    gap: 0.5rem; \
                ">
                    <ThemeToggle target_theme=Theme::Light />
                    <ThemeToggle target_theme=Theme::Dark />
                </div>
                <Show when=move || location.hash.get() != "">
                    <a style="font-size: 0.8rem;" href="/">
                        lioqing
                    </a>
                </Show>
            </div>
        </div>
        <div style=format!("\
            position: fixed; \
            width: 0px; \
            height: 0px; \
            right: 0rem; \
            top: {FRAME_WIDTH}; \
            transform: rotate(90deg); \
            overflow: visible; \
        ")>
            <div style=format!("\
                position: absolute; \
                top: 0.4rem; \
                width: calc(100vh - {FRAME_WIDTH} * 2); \
                display: flex; \
                justify-content: end; \
                align-items: center; \
            ")>
                <button
                    class="lowercase"
                    style="font-size: 0.8rem;"
                    on:click=move |_| {
                        set_background_simulated.set(background_simulated.get().toggle());
                    }
                >
                    {move || view! {
                        <i class={match background_simulated.get().is_simulated() {
                            true => "ph-fill ph-square",
                            false => "ph ph-square",
                        }}/>
                    }}
                    simulate_background
                </button>
            </div>
        </div>
        <footer style=format!("\
            position: fixed; \
            bottom: 0; \
            left: {FRAME_WIDTH}; \
            right: {FRAME_WIDTH}; \
            width: calc(100vw - {FRAME_WIDTH} * 2); \
            height: {FRAME_WIDTH}; \
            display: flex; \
            justify-content: space-between; \
            align-items: center; \
        ")>
            <p style="padding-left: 0.6rem; font-size: 0.8rem;">
                {format!("© {}, Lio Qing.", chrono::Local::now().year())}
            </p>
            <p style="padding-right: 0.2rem; font-size: 0.8rem;">
                {move || {
                    let time = chrono::DateTime::from_timestamp_millis(timestamp.get() as i64)
                        .map_or(
                            "loading...".to_string(),
                            |dt| dt
                                .with_timezone(
                                    &chrono::FixedOffset::east_opt(8 * 3600)
                                        .expect("hk timezone")
                                )
                                .format("%Y-%m-%d %H:%M:%S")
                                .to_string()
                        );
                    format!("{time} @ hk")
                }}
            </p>
        </footer>
    }
}

#[component]
fn Navigation(hash: &'static str, text: &'static str) -> impl IntoView {
    let location = use_location();
    let (theme, _) = use_theme();
    let (hovered, set_hovered) = signal(false);

    view! {
        <a
            class="lowercase"
            href=format!("/#{hash}")
            on:pointerenter=move |_| set_hovered.set(true)
            on:pointerleave=move |_| set_hovered.set(false)
        >
            {move || match location.hash.get() == format!("#{hash}") {
                true => view! {
                    <i class="ph ph-hash"/>
                }.into_any(),
                false => view! {
                    <div style=move || {
                        let color = match hovered.get() {
                            true => theme.get().foreground_color(),
                            false => "transparent",
                        };
                        format!("\
                            color: {color}; \
                            transition: {DEFAULT_TRANSITION}; \
                            height: 1em; \
                        ")
                    }>
                        <i class="ph ph-caret-right"/>
                    </div>
                }.into_any(),
            }}
            {text}
        </a>
    }
}

#[component]
fn ThemeToggle(target_theme: Theme) -> impl IntoView {
    let (theme, set_theme) = use_theme();

    view! {
        <button
            class="lowercase"
            style="font-size: 0.8rem;"
            on:click=move |_| set_theme.set(target_theme)
        >
            {move || view! {
                <i class={match theme.get() == target_theme {
                    true => "ph-fill ph-square",
                    false => "ph ph-square",
                }}/>
            }}
            {target_theme.as_str().to_lowercase()}
        </button>
    }
}
