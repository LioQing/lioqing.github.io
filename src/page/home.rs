use leptos::prelude::*;

use crate::consts::FRAME_WIDTH;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div style=format!("\
            display: flex; \
            flex-direction: column; \
            justify-content: end; \
            align-items: start; \
            min-height: calc(100% - {FRAME_WIDTH} * 2); \
            padding: 2rem; \
        ")>
            <h1>Lio Qing</h1>
            <div style="padding-left: 0.5em">
                <p>Software Engineer</p>
                <div style="height: 2rem"/>
                <p>
                    Year 4 Bachelor of Engineering in Computer Science
                </p>
                <p>
                    The University of Hong Kong
                </p>
                <p>
                    Hong Kong
                </p>
            </div>
            <div style="height: 2rem"/>
            <div style="\
                display: flex; \
                flex-direction: row; \
                justify-content: start; \
                flex-flow: row wrap; \
                gap: 1em; \
            ">
                <PageButton hash="skills" text="Skills"/>
                <PageButton hash="projects" text="Projects"/>
                <PageButton hash="experiences" text="Experiences"/>
                <a
                    href="https://github.com/LioQing"
                    target="_blank"
                    style="padding: 0.2rem;"
                >
                    <i class="ph ph-github-logo" style="font-size: 2em;"/>
                </a>
                <a
                    href="https://www.linkedin.com/in/lioqyz"
                    target="_blank"
                    style="padding: 0.2rem;"
                >
                    <i class="ph ph-linkedin-logo" style="font-size: 2em;"/>
                </a>
            </div>
        </div>
    }
}

#[component]
pub fn PageButton(hash: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <a href=format!("/#{hash}")>
            <h3>
                {text}
            </h3>
        </a>
    }
}
