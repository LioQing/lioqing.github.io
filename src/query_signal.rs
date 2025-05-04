use std::str::FromStr;

use leptos::{prelude::*, reactive::wrappers::write::SignalSetter};

/// Setting [`query_signal`] somehow duplicates the '#' sign.
pub fn query_signal<T: FromStr + ToString + PartialEq + Send + Sync + Clone>(
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
