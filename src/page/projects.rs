use std::{fmt::Display, str::FromStr, sync::OnceLock};

use chrono::Month;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use itertools::Itertools;
use leptos::{prelude::*, reactive::wrappers::write::SignalSetter};
use leptos_router::hooks::use_url;
use leptos_use::{use_clipboard, use_element_size, UseClipboardReturn, UseElementSizeReturn};

use crate::{
    consts::{DEFAULT_TRANSITION, FRAME_BACKDROP_FILTER, FRAME_WIDTH, SPACER_HEIGHT},
    page::{Content, Spacer},
    state::hooks::use_theme,
};

macro_rules! tags {
    ($($tag:ident),*) => {
        Tags::from_bits_retain($(
            Tags::$tag.bits()
        )|*)
    };
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProjectTime {
    year: u16,
    month: Month,
}

struct Project {
    title: &'static str,
    tags: Tags,
    time: ProjectTime,
    subtitle: &'static str,
    description: &'static str,
    image_url: Option<&'static str>,
    links: &'static [(&'static str, &'static str)],
}

const PROJECTS: &[Project] = &[
    Project {
        title: "WGPU 3D Gaussian Splatting Viewer App",
        tags: tags!(
            FEATURED,
            RUST,
            WEB_APP,
            DESKTOP_APP,
            WEB_ASSEMBLY,
            COMPUTER_GRAPHICS
        ),
        time: ProjectTime {
            year: 2025,
            month: Month::January,
        },
        subtitle: "3D Gaussian Splatting Viewer made in WGPU",
        description: "\
            I made this viewer for my work as a student research assistant at Innovation Wing of \
            the University of Hong Kong. It is built with WGPU, a Rust library for WebGPU, and is \
            used to visualize and edit 3D Gaussian Splatting - a cutting edge 3D reconstruction \
            technique.\
        ",
        image_url: Some("wgpu-3dgs-viewer-app.png"),
        links: &[
            ("GitHub", "https://github.com/lioqing/wgpu-3dgs-viewer-app"),
            ("Website", "https://lioqing.com/wgpu-3dgs-viewer-app"),
            ("Crates.io", "https://crates.io/crates/wgpu-3dgs-viewer"),
        ],
    },
    Project {
        title: "Simple RL Driver",
        tags: tags!(
            FEATURED,
            PYTHON,
            GAME,
            DESKTOP_APP,
            ARTIFICIAL_INTELLIGENCE,
            COMPUTER_GRAPHICS
        ),
        time: ProjectTime {
            year: 2024,
            month: Month::November,
        },
        subtitle: "The Successor of Race AI",
        description: "\
            This is a project I made to teach reinforcement learning to students from my \
            secondary school. It is built on top of my previous project Race AI, with a more \
            user-friendly interface and more features.\
        ",
        image_url: Some("simple-rl-driver.png"),
        links: &[("GitHub", "https://github.com/lioqing/simple-rl-driver")],
    },
    Project {
        title: "Talk Your Way Out",
        tags: tags!(
            FEATURED,
            PYTHON,
            CSHARP,
            GAME,
            ARTIFICIAL_INTELLIGENCE,
            COMPUTER_GRAPHICS
        ),
        time: ProjectTime {
            year: 2024,
            month: Month::April,
        },
        subtitle: "A Game with Large Language Model Driven Non-Player Characters",
        description: "\
            This is a game I made when I was the leader of the GenNarrator AI student interest \
            group at the University of Hong Kong. Players talk with non-player characters driven \
            by large language models in real-time to solve puzzles and progress in the game.\
        ",
        image_url: Some("talk-your-way-out.png"),
        links: &[(
            "InnoWing",
            "https://innoacademy.engg.hku.hk/talk_your_way_out/",
        )],
    },
    Project {
        title: "Chat Conductor",
        tags: tags!(PYTHON, TYPESCRIPT, WEB_APP, ARTIFICIAL_INTELLIGENCE),
        time: ProjectTime {
            year: 2023,
            month: Month::December,
        },
        subtitle: "Large Language Model Pipeline Prototyping Web Application",
        description: "\
            A web application for simple containerized generative AI pipeline prototyping, \
            with support for popular large language model services such as OpenAI ChatGPT, \
            Google Gemini Pro.\
        ",
        image_url: Some("chat-conductor.png"),
        links: &[("GitHub", "https://github.com/LioQing/chat-conductor")],
    },
    Project {
        title: "Code Shelf",
        tags: tags!(FEATURED, PYTHON, TYPESCRIPT, WEB_APP, COMPUTER_GRAPHICS),
        time: ProjectTime {
            year: 2023,
            month: Month::August,
        },
        subtitle: "Visualization Focused Tutorial Website",
        description: "\
            A coding tutorial site I created for learning computer science concepts, \
            with animations and interactive components for better learning experience.\
        ",
        image_url: Some("code-shelf.png"),
        links: &[
            ("GitHub", "https://github.com/LioQing/code-shelf"),
            ("Website", "https://lioqing.com/code-shelf"),
        ],
    },
    Project {
        title: "Delivery in a Heartbeat",
        tags: tags!(CSHARP, GAME),
        time: ProjectTime {
            year: 2023,
            month: Month::June,
        },
        subtitle: "Browser Game for a Game Jam",
        description: "\
            A mini game I made with my girlfriend in Unity for a game jam. We completed it \
            within 48 hours.\
        ",
        image_url: Some("delivery-in-a-heartbeat.png"),
        links: &[
            (
                "GitHub",
                "https://github.com/LioQing/delivery-in-a-heartbeat",
            ),
            (
                "Itch.io",
                "https://lio-qing.itch.io/delivery-in-a-heartbeat",
            ),
        ],
    },
    Project {
        title: "Race AI",
        tags: tags!(
            PYTHON,
            GAME,
            DESKTOP_APP,
            ARTIFICIAL_INTELLIGENCE,
            COMPUTER_GRAPHICS
        ),
        time: ProjectTime {
            year: 2023,
            month: Month::April,
        },
        subtitle: "Fun Machine Learning Project",
        description: "\
            A study project after my completion of the Machine Learning specialization \
            certificate to solidify my knowledge. Implement self-learning AI drivers without \
            additional help of any machine learning library.\
        ",
        image_url: Some("race-ai.png"),
        links: &[("GitHub", "https://github.com/LioQing/RaceAI")],
    },
    Project {
        title: "HKU Timetable Viewer",
        tags: tags!(FEATURED, TYPESCRIPT, WEB_APP),
        time: ProjectTime {
            year: 2022,
            month: Month::July,
        },
        subtitle: "A Timetable Viewer for University of Hong Kong Students",
        description: "\
            Back then, the university did not have a timetable planner/viewer, so I made one. \
            It was a great help for me and my schoolmates, but now the university has its own \
            so it is much less used.\
        ",
        image_url: Some("hku-timetable-viewer.png"),
        links: &[
            ("GitHub", "https://github.com/LioQing/hku-timetable-viewer"),
            ("Website", "https://lioqing.com/hku-timetable-viewer"),
        ],
    },
    Project {
        title: "Custom Float Calculator",
        tags: tags!(RUST, WEB_APP, WEB_ASSEMBLY),
        time: ProjectTime {
            year: 2022,
            month: Month::June,
        },
        subtitle: "Visualizations for Floating Point Numbers",
        description: "\
            This is the first project I made with Rust and WebAssembly. It was a great learning \
            experience learning Rust, as well as solidifying my knowledge in floating point \
            numbers.\
        ",
        image_url: Some("custom-float-calculator.png"),
        links: &[
            (
                "GitHub",
                "https://github.com/LioQing/custom-float-calculator",
            ),
            ("Website", "https://lioqing.com/custom-float-calculator"),
            ("Crates.io", "https://crates.io/crates/float-format"),
        ],
    },
    Project {
        title: "Personal Website 2022",
        tags: tags!(TYPESCRIPT, WEB_APP),
        time: ProjectTime {
            year: 2022,
            month: Month::June,
        },
        subtitle: "The First Version of This Website",
        description: "\
            This is the first version of my personal website. I built it right after I finished \
            the Full Stack open online course, which was really helpful.\
        ",
        image_url: Some("personal-website.png"),
        links: &[
            ("GitHub", "https://github.com/LioQing/personal-website-2022"),
            ("Website", "https://lioqing.com/personal-website-2022"),
        ],
    },
    Project {
        title: "Tiny Terminal User Interface",
        tags: tags!(CPLUSPLUS, DESKTOP_APP),
        time: ProjectTime {
            year: 2022,
            month: Month::May,
        },
        subtitle: "Terminal User Interface Library",
        description: "\
            A lightweight TUI library I wrote to assist in my university project. Used many \
            Linux system libraries for terminal input and output.\
        ",
        image_url: Some("ttui.png"),
        links: &[("GitHub", "https://github.com/LioQing/ttui")],
    },
    Project {
        title: "Everything Daily",
        tags: tags!(CSHARP, DESKTOP_APP),
        time: ProjectTime {
            year: 2022,
            month: Month::January,
        },
        subtitle: "Utility Desktop Application for Recording Daily Routines",
        description: "\
            A Windows application for me to record my daily activities where I can create \
            activity type, add records, delete records all using the app.\
        ",
        image_url: Some("everything-daily.png"),
        links: &[("GitHub", "https://github.com/LioQing/Everything-Daily")],
    },
    Project {
        title: "The Tank Arena",
        tags: tags!(
            FEATURED,
            CPLUSPLUS,
            GAME,
            COMPUTER_GRAPHICS,
            ARTIFICIAL_INTELLIGENCE
        ),
        time: ProjectTime {
            year: 2021,
            month: Month::July,
        },
        subtitle: "Tank Game with AI",
        description: "\
            A game built in a custom game engine with an architecture optimized for data \
            intensive calculation. Implement all components including AI, level, physics from \
            scratch with a single graphics library SFML. It is one of my first major project.\
        ",
        image_url: Some("the-tank-arena.png"),
        links: &[("GitHub", "https://github.com/LioQing/The-Tank-Arena")],
    },
    Project {
        title: "Bro Well 兄井",
        tags: tags!(FEATURED, ILLUSTRATOR, BLENDER, GRAPHIC),
        time: ProjectTime {
            year: 2023,
            month: Month::March,
        },
        subtitle: "Tribute to My Favorite Restaurant",
        description: "\
            A popular restaurant around my secondary school, I frequently go there during my \
            high school years. Unfortunately, it has closed down.\
        ",
        image_url: Some("bro-well.png"),
        links: &[],
    },
    Project {
        title: "Mountain Hike",
        tags: tags!(ILLUSTRATOR, BLENDER, GRAPHIC),
        time: ProjectTime {
            year: 2023,
            month: Month::March,
        },
        subtitle: "Hiking Logo for My Instagram",
        description: "\
            I like hiking. Unlike going to gym or working out in a fixed place, I enjoy going to \
            places and explore the nature.\
        ",
        image_url: Some("mountain-hike.png"),
        links: &[],
    },
    Project {
        title: "St. Paul's College Biology Society Logo",
        tags: tags!(FEATURED, ILLUSTRATOR, GRAPHIC),
        time: ProjectTime {
            year: 2020,
            month: Month::October,
        },
        subtitle: "The Biology Society Logo for My Secondary School",
        description: "\
            I was not good at biology, but the lessons were very interesting and I really like it. \
            I am also a part of the Biology Society committee.\
        ",
        image_url: Some("bio-soc.png"),
        links: &[],
    },
    Project {
        title: "St. Paul's College Computer Society Logo",
        tags: tags!(ILLUSTRATOR, GRAPHIC),
        time: ProjectTime {
            year: 2020,
            month: Month::October,
        },
        subtitle: "The Computer Society Logo for My Secondary School",
        description: "\
            I still have very close friends from my secondary school, especially those who were \
            in the Computer Society committee with me.\
        ",
        image_url: Some("comp-soc.png"),
        links: &[],
    },
    Project {
        title: "St. Paul's College Robotics Team Logo",
        tags: tags!(ILLUSTRATOR, GRAPHIC),
        time: ProjectTime {
            year: 2020,
            month: Month::September,
        },
        subtitle: "The Robotics Team Logo for My Secondary School",
        description: "\
            Along with the Computer Society logo, I also designed the logo for Robotics Team, even \
            though I was not a member of it.\
        ",
        image_url: Some("robotics.png"),
        links: &[],
    },
    Project {
        title: "Speed Cubing 3x3 World Record Poster",
        tags: tags!(ILLUSTRATOR, GRAPHIC),
        time: ProjectTime {
            year: 2019,
            month: Month::January,
        },
        subtitle: "Fun Little Poster I made for the Rubik's Cube World Record",
        description: "\
            I was really into Rubik's Cube in my junior years of secondary school, this was a \
            little project I did for fun.\
        ",
        image_url: Some("cubing-wr.png"),
        links: &[],
    },
    Project {
        title: "Solve It Rubik's Cube Poster",
        tags: tags!(PHOTOSHOP, GRAPHIC),
        time: ProjectTime {
            year: 2018,
            month: Month::May,
        },
        subtitle: "Mobile Wallpaper",
        description: "\
            I liked Rubik's Cube so much that I made a wallpaper for my own smart phone. It was \
            one of the few times I actually do a digital art.\
        ",
        image_url: Some("solve-it.png"),
        links: &[],
    },
];

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
    pub struct Tags: u32 {
        const FEATURED                  = 1 << 0;

        const RUST                      = 1 << 1;
        const PYTHON                    = 1 << 2;
        const CPLUSPLUS                 = 1 << 3;
        const CSHARP                    = 1 << 4;
        const TYPESCRIPT                = 1 << 5;

        const ILLUSTRATOR               = 1 << 6;
        const PHOTOSHOP                 = 1 << 7;
        const BLENDER                   = 1 << 8;

        const WEB_APP                   = 1 << 9;
        const DESKTOP_APP               = 1 << 10;
        const MOBILE_APP                = 1 << 11;
        const GAME                      = 1 << 12;
        const GRAPHIC                   = 1 << 13;

        const ARTIFICIAL_INTELLIGENCE   = 1 << 14;
        const WEB_ASSEMBLY              = 1 << 15;
        const COMPUTER_GRAPHICS         = 1 << 16;

        const PROGRAMMING_LANGUAGES = Self::RUST.bits()
            | Self::PYTHON.bits()
            | Self::CPLUSPLUS.bits()
            | Self::CSHARP.bits()
            | Self::TYPESCRIPT.bits();

        const COMPUTER_SOFTWARES = Self::ILLUSTRATOR.bits()
            | Self::PHOTOSHOP.bits()
            | Self::BLENDER.bits();

        const APPLICATIONS = Self::WEB_APP.bits()
            | Self::DESKTOP_APP.bits()
            | Self::MOBILE_APP.bits()
            | Self::GAME.bits()
            | Self::GRAPHIC.bits();

        const TECHNOLOGIES = Self::ARTIFICIAL_INTELLIGENCE.bits()
            | Self::WEB_ASSEMBLY.bits()
            | Self::COMPUTER_GRAPHICS.bits();
    }
}

impl Tags {
    pub fn names(self) -> impl Iterator<Item = &'static str> {
        self.iter().map(|tag| match tag {
            Tags::FEATURED => "Featured",
            Tags::RUST => "Rust",
            Tags::PYTHON => "Python",
            Tags::CPLUSPLUS => "C++",
            Tags::CSHARP => "C#",
            Tags::TYPESCRIPT => "TypeScript",
            Tags::ILLUSTRATOR => "Illustrator",
            Tags::PHOTOSHOP => "Photoshop",
            Tags::BLENDER => "Blender",
            Tags::WEB_APP => "Web",
            Tags::DESKTOP_APP => "Desktop",
            Tags::MOBILE_APP => "Mobile",
            Tags::GAME => "Game",
            Tags::GRAPHIC => "Graphics",
            Tags::ARTIFICIAL_INTELLIGENCE => "Artificial Intelligence",
            Tags::WEB_ASSEMBLY => "WebAssembly",
            Tags::COMPUTER_GRAPHICS => "Computer Graphics",
            _ => panic!("Unknown tag: {tag:?}"),
        })
    }

    pub fn icons(self) -> impl Iterator<Item = &'static str> {
        self.iter().map(|tag| match tag {
            Tags::FEATURED => "ph-fill ph-star",
            Tags::RUST => "devicon-rust-original",
            Tags::PYTHON => "devicon-python-plain",
            Tags::CPLUSPLUS => "devicon-cplusplus-plain",
            Tags::CSHARP => "devicon-csharp-plain",
            Tags::TYPESCRIPT => "devicon-typescript-plain",
            Tags::ILLUSTRATOR => "devicon-illustrator-plain",
            Tags::PHOTOSHOP => "devicon-photoshop-plain",
            Tags::BLENDER => "devicon-blender-original",
            Tags::WEB_APP => "ph-fill ph-browsers",
            Tags::DESKTOP_APP => "ph-fill ph-desktop",
            Tags::MOBILE_APP => "ph-fill ph-device-mobile",
            Tags::GAME => "ph-fill ph-game-controller",
            Tags::GRAPHIC => "ph-fill ph-compass-tool",
            Tags::ARTIFICIAL_INTELLIGENCE => "ph-fill ph-robot",
            Tags::WEB_ASSEMBLY => "devicon-wasm-original",
            Tags::COMPUTER_GRAPHICS => "ph-fill ph-graphics-card",
            _ => panic!("Unknown tag: {tag:?}"),
        })
    }
}

impl FromStr for Tags {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(Tags::from_bits_truncate)
    }
}

impl Display for Tags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bits().fmt(f)
    }
}

impl Default for Tags {
    fn default() -> Self {
        Tags::FEATURED
    }
}

fn get_projects(search: &str, tags: Tags) -> impl Iterator<Item = (YearPosition, usize)> {
    let projects = match search {
        "" => (0..PROJECTS.len())
            .sorted_by_key(|i| &PROJECTS[*i].time)
            .collect::<Vec<_>>(),
        search => {
            static MATCHER: OnceLock<SkimMatcherV2> = OnceLock::new();
            let matcher = MATCHER.get_or_init(|| SkimMatcherV2::default().ignore_case());
            let mut matched = (0..PROJECTS.len())
                .map(|i| {
                    (
                        matcher
                            .fuzzy_indices(PROJECTS[i].title, search)
                            .map(|(score, _)| score)
                            .unwrap_or(0),
                        i,
                    )
                })
                .collect::<Vec<_>>();

            let threshold = matched.iter().map(|(score, _)| score).max().unwrap_or(&0) - 2;

            matched.sort_by_key(|(_, i)| &PROJECTS[*i].time);

            matched
                .into_iter()
                .filter(move |(score, _)| *score >= threshold)
                .map(|(_, i)| i)
                .collect::<Vec<_>>()
        }
    }
    .into_iter()
    .rev()
    .filter(move |i| PROJECTS[*i].tags.intersects(tags));

    std::iter::once(None)
        .chain(projects.map(|i| Some((PROJECTS[i].time.year, i))))
        .chain(std::iter::once(None))
        .tuple_windows()
        .map(|(prev, curr, next)| match (prev, curr, next) {
            (None, Some((curr_year, curr)), Some((other_year, _)))
            | (Some((other_year, _)), Some((curr_year, curr)), None)
                if curr_year != other_year =>
            {
                (YearPosition::Single, curr)
            }
            (None, Some((_, curr)), Some(_)) => (YearPosition::First, curr),
            (Some(_), Some((_, curr)), None) => (YearPosition::Last, curr),
            (Some((prev_year, _)), Some((curr_year, curr)), Some((next_year, _))) => {
                let pos = if prev_year == curr_year && curr_year == next_year {
                    YearPosition::Middle
                } else if curr_year == next_year {
                    YearPosition::First
                } else if prev_year == curr_year {
                    YearPosition::Last
                } else {
                    YearPosition::Single
                };
                (pos, curr)
            }
            (None, Some((_, curr)), None) => (YearPosition::Single, curr),
            _ => unreachable!(),
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YearPosition {
    First,
    Middle,
    Last,
    Single,
}

/// Setting [`query_signal`] somehow duplicates the '#' sign.
fn query_signal<T: FromStr + ToString + PartialEq + Send + Sync + Clone>(
    key: &'static str,
) -> (Memo<Option<T>>, SignalSetter<Option<T>>) {
    let (query, set_query) = leptos_router::hooks::query_signal(key);

    Effect::watch(
        move || query.get(),
        move |_, _, _| {
            let window = web_sys::window().expect("window");
            let url =
                web_sys::Url::new(window.location().href().expect("href").as_str()).expect("url");

            window
                .history()
                .expect("history")
                .replace_state_with_url(
                    &leptos_router::location::State::default().to_js_value(),
                    "",
                    Some(url.href().replace("##", "#").as_str()),
                )
                .expect("push state with url");
        },
        false,
    );

    (query, set_query)
}

#[component]
pub fn Projects() -> impl IntoView {
    let (search, _) = query_signal::<String>("search");
    let (tags, _) = query_signal::<Tags>("tags");
    let projects = Memo::new(move |_| {
        get_projects(
            &search.get().unwrap_or_default(),
            tags.get().unwrap_or_default(),
        )
        .collect::<Vec<_>>()
    });

    view! {
        <Content
            icon="ph-floppy-disk"
            title="Projects"
            subtitle="Softwares I built & graphics I designed."
        >
            <div style="height: 2.4rem;"/>
            <div style="padding-left: 0.4em">
                <Tools/>
            </div>
            <Spacer/>
            {move || projects.with(|projects| projects
                .iter()
                .enumerate()
                .map(|(index, (pos, i))| {
                    let project = &PROJECTS[*i];
                    view! {
                        <Show when=move || (index > 0)>
                            <Spacer/>
                        </Show>
                        <ItemWithTimeline project=project pos=*pos/>
                    }.into_any()
                })
                .collect_view()
            )}
        </Content>
    }
}

#[component]
fn Tools() -> impl IntoView {
    let search_label_ref = NodeRef::<leptos::html::P>::new();
    let UseElementSizeReturn {
        width: search_label_width,
        ..
    } = use_element_size(search_label_ref);
    let tags_label_ref = NodeRef::<leptos::html::P>::new();
    let UseElementSizeReturn {
        width: tags_label_width,
        ..
    } = use_element_size(tags_label_ref);
    let (search, set_search) = query_signal::<String>("search");
    let (tags, set_tags) = query_signal::<Tags>("tags");

    let get_tags = move || tags.get().unwrap_or_default();

    let set_tags = move |tags: Tags| {
        if get_tags() == tags {
            return;
        }

        set_tags.set((tags != Tags::default()).then_some(tags));
    };

    let toggle_tag = move |tag: Tags| {
        set_tags(get_tags() ^ tag);
    };

    view! {
        <div style="position: relative;">
            <input
                type="text"
                autocomplete="off"
                on:input=move |e| set_search.set(match event_target_value(&e).as_str() {
                    "" => None,
                    search => Some(search.to_string()),
                })
                style=move || {
                    let width = search_label_width.get();
                    format!("\
                        width: 20rem; \
                        mask-image: linear-gradient( \
                            to top, \
                            black 0% calc(100% - 1px), \
                            transparent calc(100% - 1px) 100% \
                        ), linear-gradient( \
                            to left, \
                            black 0% calc(100% - 1.5rem - {width}px), \
                            transparent calc(100% - 1.5rem - {width}px) 100% \
                        ), linear-gradient( \
                            to right, \
                            black 0% 0.5rem, \
                            transparent 0.5rem 100% \
                        ) \
                    ")
                }
            />
            <p
                node_ref=search_label_ref
                style="\
                    position: absolute; \
                    left: 1rem; \
                    top: -0.4rem; \
                    font-size: 0.8rem; \
                    max-width: 19rem; \
                    white-space: nowrap; \
                    overflow: hidden; \
                    text-overflow: ellipsis; \
                "
            >
                {move || format!(
                    "search={}",
                    &search.get().unwrap_or_default()
                )}
            </p>
        </div>
        <Spacer small=true/>
        <div style="position: relative;">
            <div style=move || {
                let width = tags_label_width.get();
                format!("\
                    display: flex; \
                    flex-direction: column; \
                    gap: 0.4rem; \
                    padding: 1rem 0.8rem; \
                    border: 1px solid; \
                    backdrop-filter: {FRAME_BACKDROP_FILTER}; \
                    transition: {DEFAULT_TRANSITION}; \
                    mask-image: linear-gradient( \
                        to top, \
                        black 0% calc(100% - 1px), \
                        transparent calc(100% - 1px) 100% \
                    ), linear-gradient( \
                        to left, \
                        black 0% calc(100% - 1.5rem - {width}px), \
                        transparent calc(100% - 1.5rem - {width}px) 100% \
                    ), linear-gradient( \
                        to right, \
                        black 0% 0.5rem, \
                        transparent 0.5rem 100% \
                    ) \
                ")
            }>
                <div style="display: flex; flex-direction: row; gap: 0.4rem;">
                    <button class="lowercase" on:click=move |_| set_tags(Tags::all())>
                        {move || view! {
                            <i class={match get_tags() == Tags::all() {
                                true => "ph-fill ph-square",
                                false => "ph ph-square",
                            }}/>
                        }}
                        all
                    </button>
                    <button class="lowercase" on:click=move |_| set_tags(Tags::empty())>
                        {move || view! {
                            <i class={match get_tags() == Tags::empty() {
                                true => "ph-fill ph-square",
                                false => "ph ph-square",
                            }}/>
                        }}
                        none
                    </button>
                    <button class="lowercase" on:click=move |_| toggle_tag(Tags::FEATURED)>
                        {move || view! {
                            <i class={match get_tags().contains(Tags::FEATURED) {
                                true => "ph-fill ph-square",
                                false => "ph ph-square",
                            }}/>
                        }}
                        featured
                    </button>
                </div>
                <div style="display: flex; flex-direction: row; gap: 0.4rem;">
                    <div style="display: flex; flex-direction: column; gap: 0.4rem;">
                        {[
                            ("programming_languages", Tags::PROGRAMMING_LANGUAGES),
                            ("computer_softwares", Tags::COMPUTER_SOFTWARES),
                            ("applications", Tags::APPLICATIONS),
                            ("technologies", Tags::TECHNOLOGIES),
                        ]
                            .iter()
                            .map(|(category, tags)| view! {
                                <button class="lowercase" on:click=move |_| set_tags(*tags)>
                                    {move || view! {
                                        <i class={match get_tags().contains(*tags) {
                                            true => "ph-fill ph-square",
                                            false => "ph ph-square",
                                        }}/>
                                    }}
                                    {*category}
                                </button>
                            })
                            .collect_view()
                        }
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 0.4rem;">
                        {[
                            Tags::PROGRAMMING_LANGUAGES,
                            Tags::COMPUTER_SOFTWARES,
                            Tags::APPLICATIONS,
                            Tags::TECHNOLOGIES,
                        ]
                            .iter()
                            .map(|tags| view! {
                                <div style="display: flex; flex-direction: row; gap: 0.4rem;">
                                    {tags.iter().map(|tag| view! {
                                        <button class="lowercase" on:click=move |_| toggle_tag(tag)>
                                            {move || view! {
                                                <i class={match get_tags().contains(tag) {
                                                    true => "ph-fill ph-square",
                                                    false => "ph ph-square",
                                                }}/>
                                            }}
                                            {tag.names().next().unwrap().replace(" ", "_")}
                                        </button>
                                    }).collect_view()}
                                </div>
                            })
                            .collect_view()
                        }
                    </div>
                </div>
            </div>
            <p
                node_ref=tags_label_ref
                style="\
                    position: absolute; \
                    left: 1rem; \
                    top: -0.4rem; \
                    font-size: 0.8rem; \
                    max-width: 19rem; \
                    white-space: nowrap; \
                    overflow: hidden; \
                    text-overflow: ellipsis; \
                "
            >
                {move || format!(
                    "tags={}",
                    match tags.get() {
                        Some(tags) => leptos_router::location::Url::escape(&tags.to_string()),
                        None => String::new(),
                    }
                )}
            </p>
        </div>
    }
}

#[component]
fn ItemWithTimeline(
    project: &'static Project,
    #[prop(into)] pos: Signal<YearPosition>,
) -> impl IntoView {
    let item_ref = NodeRef::<leptos::html::Div>::new();
    let UseElementSizeReturn {
        height: item_height,
        ..
    } = use_element_size(item_ref);

    view! {
        <div style="\
            display: flex; \
            flex-direction: row; \
            justify-content: start; \
            align-items: end; \
            gap: 2rem; \
            padding: 0rem 0.5rem; \
        ">
            <div style="position: relative; width: 1px; height: 1px;">
                {move || match pos.get() {
                    YearPosition::Middle => view! {
                        <div style=move || {
                            let height = item_height.get();
                            format!("\
                                position: absolute; \
                                bottom: 0; \
                                height: calc({height}px + {SPACER_HEIGHT}); \
                                border-left: 1px solid; \
                            ")
                        }/>
                    }.into_any(),
                    YearPosition::Last => view! {
                        <div style=move || {
                            let height = item_height.get();
                            format!("\
                                position: absolute; \
                                bottom: calc({height}px * 0.5); \
                                height: calc({height}px * 0.5 + {SPACER_HEIGHT}); \
                                border-left: 1px solid; \
                            ")
                        }/>
                        <svg
                            width="2em"
                            height="2em"
                            style=move || {
                                let height = item_height.get();
                                format!("\
                                    position: absolute; \
                                    bottom: calc({height}px * 0.5 - 1.5em); \
                                    left: -1em; \
                                ")
                            }
                        >
                            <circle
                                cx="1em"
                                cy="1em"
                                r="0.5em"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1"
                            />
                        </svg>
                    }.into_any(),
                    YearPosition::First | YearPosition::Single => view! {
                        <div style="\
                            position: absolute; \
                            bottom: 0; \
                            transform: translateX(-50%) rotate(-90deg) translateX(50%); \
                        ">
                            <p style="top: -0.5em; left: 1em;">{project.time.year}</p>
                        </div>
                    }.into_any(),
                }}
            </div>
            <Item
                node_ref=item_ref
                title=project.title
                tags=project.tags
                time=&project.time
                subtitle=project.subtitle
                description=project.description
                image_url=project.image_url
                links=project.links
            />
        </div>
    }
}

#[component]
fn Item(
    title: &'static str,
    tags: Tags,
    time: &'static ProjectTime,
    subtitle: &'static str,
    description: &'static str,
    image_url: Option<&'static str>,
    #[prop(optional, into)] links: &'static [(&'static str, &'static str)],
    #[prop(optional, into)] node_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    let image_caption_ref = NodeRef::<leptos::html::P>::new();
    let UseElementSizeReturn {
        width: image_caption_width,
        ..
    } = use_element_size(image_caption_ref);
    let UseClipboardReturn {
        is_supported: is_copy_supported,
        copied,
        copy,
        ..
    } = use_clipboard();
    let url = use_url();
    let (theme, _) = use_theme();
    let (project, set_project) = query_signal::<String>("project");

    let project_url = Memo::new(move |_| {
        let origin = url.with(|url| url.origin().to_string());
        let title_escaped = leptos_router::location::Url::escape(title);
        format!("{origin}/?project={title_escaped}#projects")
    });

    view! {
        <div
            node_ref=node_ref
            style="\
                display: flex; \
                flex-direction: column; \
                align-items: start; \
            "
        >
            <button
                on:click=move |_| set_project.set(Some(title.to_string()))
                style="margin-bottom: 0.4em;"
            >
                <h3>{title}</h3>
            </button>
            <div style="\
                display: flex; \
                flex-direction: row; \
                flex-wrap: wrap; \
                justify-content: start; \
                align-items: center; \
                gap: 0.4em; \
                padding-left: 0.4em; \
            ">
                {Itertools::intersperse_with(tags
                    .icons()
                    .zip(tags.names())
                    .map(|(icon, name)| view! {
                        <i class={icon} title={name}/>
                        <p>{name}</p>
                    }.into_any()),
                    || view! { | }.into_any(),
                ).collect_view()}
            </div>
        </div>
        <Show when=move || project.get() == Some(title.to_string())>
            <div
                on:click=move |_| set_project.set(None)
                style="\
                    position: absolute; \
                    top: 0; \
                    left: 0; \
                    width: 100%; \
                    height: 100%; \
                    display: flex; \
                    justify-content: center; \
                    align-items: center; \
                "
            >
                <dialog
                    open=move || project.get() == Some(title.to_string())
                    on:close=move |_| set_project.set(None)
                    on:click=move |event| event.stop_propagation()
                    style=move || {
                        let background_color = theme.get().background_color().to_string() + "80";
                        format!("\
                            position: relative; \
                            max-width: min(60vw, 800em); \
                            max-height: min(min(80vh, 600em), 100vh - {FRAME_WIDTH} * 4); \
                            border: 1px solid; \
                            backdrop-filter: {FRAME_BACKDROP_FILTER}; \
                            background-color: {background_color}; \
                            color: inherit; \
                            overflow-y: auto; \
                        ")
                    }
                >
                    <div style=format!("\
                        position: absolute; \
                        top: 0; \
                        width: 100%; \
                        height: {FRAME_WIDTH}; \
                        border-bottom: 1px solid; \
                        font-size: 0.8rem; \
                    ")>
                        <div style="\
                            display: flex; \
                            flex-direction: row; \
                            justify-content: space-between; \
                            align-items: center; \
                            height: 100%; \
                            padding: 0 0.8rem; \
                        ">
                            {
                                let project_url = project_url;
                                let copy = copy.clone();
                                move || match is_copy_supported.get() {
                                    true => {
                                        let project_url = project_url;
                                        let copy = copy.clone();
                                        view! {
                                            <div style="\
                                                display: flex; \
                                                flex-direction: row; \
                                                align-items: center; \
                                                gap: 0.4rem; \
                                            ">
                                                <button
                                                    on:click=move |_| copy(&project_url.get())
                                                    onmouseover="\
                                                        this.querySelector('i').style.color = ''\
                                                    "
                                                    onmouseleave="\
                                                        this.querySelector('i').style.color = \
                                                        'transparent'\
                                                    "
                                                >
                                                    {project_url}
                                                    <i
                                                        class="ph ph-copy"
                                                        style=format!("\
                                                            transition: {DEFAULT_TRANSITION}; \
                                                        ")
                                                    />
                                                </button>
                                                <Show when=move || copied.get()>
                                                    Copied.
                                                </Show>
                                            </div>
                                        }.into_any()
                                    },
                                    false => view! { {project_url} }.into_any()
                                }
                            }
                            <button
                                on:click=move |_| set_project.set(None)
                                style="padding: 0.1rem;"
                            >
                                <i class="ph ph-x"/>
                            </button>
                        </div>
                    </div>
                    <div style="\
                        overflow: auto; \
                    ">
                        <div style="\
                            display: inline-flex; \
                            flex-direction: row; \
                            justify-content: space-between; \
                            gap: 4rem; \
                            padding: 4rem; \
                        ">
                            <div style="flex: 1;">
                                <div style="\
                                    display: flex; \
                                    flex-direction: column; \
                                    align-items: start; \
                                ">
                                    <p style="padding-left: 0.2rem; padding-bottom: 1rem;">
                                        {format!("{} {}", time.month.name(), time.year)}
                                    </p>
                                    <h2>{title}</h2>
                                    <p style="padding-left: 0.2rem;">{subtitle}</p>
                                </div>
                                <Spacer/>
                                <div style="\
                                    display: flex; \
                                    flex-direction: row; \
                                    flex-wrap: wrap; \
                                    justify-content: start; \
                                    align-items: center; \
                                    gap: 0.4rem; \
                                ">
                                    {Itertools::intersperse_with(tags
                                        .icons()
                                        .zip(tags.names())
                                        .map(|(icon, name)| view! {
                                            <i class={icon} title={name}/>
                                            <p>{name}</p>
                                        }.into_any()),
                                        || view! { | }.into_any(),
                                    ).collect_view()}
                                </div>
                                <Spacer/>
                                {move || match image_url {
                                    Some(url) if tags.contains(Tags::GRAPHIC) => view! {
                                        <div style="\
                                            display: flex; \
                                            justify-content: center; \
                                            align-items: center; \
                                            width: 100%; \
                                            margin-bottom: 1rem; \
                                        ">
                                            <img
                                                src={format!(
                                                    "/assets/images/project-graphics/{url}"
                                                )}
                                                alt=title
                                                style="\
                                                    width: 100%; \
                                                    height: auto; \
                                                    max-height: 60vh; \
                                                    object-fit: contain; \
                                                "
                                            />
                                        </div>
                                    }.into_any(),
                                    _ => ().into_any(),
                                }}
                                <p style="white-space: pre-wrap; line-height: 1.5;">
                                    {description}
                                </p>
                                <Spacer/>
                                {(!links.is_empty()).then(|| view! {
                                    <div style="display: flex; flex-wrap: wrap; gap: 1rem;">
                                        {links.iter().map(|(text, url)| view! {
                                            <a
                                                href={url.to_string()}
                                                target="_blank"
                                                style="padding: 0.5rem 1rem;"
                                            >
                                                {text.to_string()}
                                                <i class="ph ph-arrow-square-out"/>
                                            </a>
                                        }.into_any()).collect_view()}
                                    </div>
                                })}
                            </div>
                            {match image_url {
                                Some(url) if !tags.contains(Tags::GRAPHIC) => view! {
                                    <div style="position: relative;">
                                        <div style=move || {
                                            let width = image_caption_width.get();
                                            format!("\
                                                display: flex; \
                                                justify-content: center; \
                                                align-items: center; \
                                                border: 1px solid; \
                                                padding: 1rem; \
                                                height: min(60vh, 20rem); \
                                                width: min(60vh, 20rem); \
                                                mask-image: linear-gradient( \
                                                    to bottom, \
                                                    black 0% calc(100% - 1px), \
                                                    transparent calc(100% - 1px) 100% \
                                                ), linear-gradient( \
                                                    to left, \
                                                    black \
                                                        0% \
                                                        calc(100% - 2rem - {width}px), \
                                                    transparent \
                                                        calc(100% - 2rem - {width}px) \
                                                        100% \
                                                ), linear-gradient( \
                                                    to right, \
                                                    black 0% 1rem, \
                                                    transparent 1rem 100% \
                                                ) \
                                            ")
                                        }>
                                            <img
                                                src={format!(
                                                    "/assets/images/project-covers/{url}"
                                                )}
                                                alt=title
                                                style="\
                                                    width: auto; \
                                                    height: 100%; \
                                                    object-fit: cover; \
                                                "
                                            />
                                        </div>
                                        <p
                                            node_ref=image_caption_ref
                                            style="\
                                                position: absolute; \
                                                left: 1.5rem; \
                                                top: calc(1.5rem + min(60vh, 20rem)); \
                                                font-size: 0.8rem; \
                                                max-width: calc(min(60vh, 20rem) - 1.5rem); \
                                                overflow: hidden; \
                                                white-space: nowrap; \
                                                text-overflow: ellipsis; \
                                            "
                                        >
                                            {url}
                                        </p>
                                    </div>
                                }.into_any(),
                                _ => ().into_any(),
                            }}
                        </div>
                    </div>
                </dialog>
            </div>
        </Show>
    }
}
