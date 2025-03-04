#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let (_, rx) = std::sync::mpsc::channel();
    lioqing::background(rx);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("unsupported target architecture: wasm32, this is a native only binary");
}
