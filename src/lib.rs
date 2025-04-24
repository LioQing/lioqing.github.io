#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod consts;
#[cfg(target_arch = "wasm32")]
mod page;
#[cfg(target_arch = "wasm32")]
pub use app::{App, AppProps};

mod state;
pub use state::*;

mod background;
pub use background::*;
