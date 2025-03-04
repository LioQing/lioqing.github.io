use leptos::prelude::*;

use crate::page::{Content, Spacer, Tags};

#[component]
pub fn Skills() -> impl IntoView {
    view! {
        <Content
            icon="ph-toolbox"
            title="Skills"
            subtitle="My passion and career as a software engineer."
        >
            <Section title="Programming Languages."/>
            <Items
                name="Rust"
                icon="rust-original"
                tags=Tags::RUST
                description="My most frequently used language, and my favorite."
            />
            <Spacer/>
            <Items
                name="Python"
                icon="python-plain"
                tags=Tags::PYTHON
                description="The popular kid in school, so obviously I used it a lot."
            />
            <Spacer/>
            <Items
                name="C++"
                icon="cplusplus-plain"
                tags=Tags::CPLUSPLUS
                description="The good old friend, the first one I learnt from my high school, but we don't talk much now."
            />
            <Spacer/>
            <Items
                name="C#"
                icon="csharp-plain"
                tags=Tags::CSHARP
                description="The first proper language I learnt, for Unity."
            />
            <Spacer/>
            <Items
                name="TypeScript"
                icon="typescript-plain"
                tags=Tags::TYPESCRIPT
                description="I followed it through my web journey, even though I am not too into front-end."
            />
            <Section title="Computer Softwares."/>
            <Items
                name="Illustrator"
                icon="illustrator-plain"
                tags=Tags::ILLUSTRATOR
                description="I fell in love with graphic design the moment I learnt about vector art."
            />
            <Spacer/>
            <Items
                name="Photoshop"
                icon="photoshop-plain"
                tags=Tags::PHOTOSHOP
                description="Having parents graduated from industrial design, I knew it since I was like five."
            />
            <Spacer/>
            <Items
                name="Blender"
                icon="blender-original"
                tags=Tags::BLENDER
                description="Tried learning many times, still can't make a gorgeous scene that I want."
            />
        </Content>
    }
}

#[component]
fn Items(
    name: &'static str,
    icon: &'static str,
    tags: Tags,
    description: &'static str,
) -> impl IntoView {
    view! {
        <a
            href=format!("/?tags={tags}#projects")
            style="\
                display: flex; \
                flex-direction: row; \
                justify-content: start; \
                align-items: center; \
                gap: 1em; \
                padding-left: 0.4em; \
                margin-bottom: 0.4em; \
            "
        >
            <i class={format!("devicon-{icon}")} style="font-size: 2em;"/>
            <h3>{name}</h3>
        </a>
        <p style="padding-left: 0.2em;">{description}</p>
    }
}

#[component]
pub fn Section(title: &'static str) -> impl IntoView {
    view! {
        <Spacer/>
        <i class="ph ph-dots-six" style="font-size: 2.4em;"/>
        <div style="height: 0.8rem;"/>
        <h3>{title}</h3>
        <div style="height: 2rem;"/>
    }
}
