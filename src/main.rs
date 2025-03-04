#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();

    leptos::mount::mount_to_body(lioqing::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    panic!("unsupported target architecture: not wasm32, this is a wasm32 only binary");
}
