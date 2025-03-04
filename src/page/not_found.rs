use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div style="\
            display: flex; \
            flex-direction: column; \
            justify-content: center; \
            align-items: center; \
            height: 100%; \
            gap: 0.5em; \
        ">
            <p>
                Page not found
            </p>
            <a href="/">
                <i class="ph ph-arrow-u-down-left" size="1em"/>
                Back
            </a>
        </div>
    }
}
