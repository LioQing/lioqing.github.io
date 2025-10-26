pub fn init(filter: log::LevelFilter) {
    struct Logger {
        filter: log::LevelFilter,
    }

    impl Logger {
        fn new(filter: log::LevelFilter) -> Self {
            Self { filter }
        }
    }

    impl log::Log for Logger {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            const CRATES_AT_INFO_LEVEL: &[&str] = &["naga", "wgpu_core", "wgpu_hal"];

            if CRATES_AT_INFO_LEVEL
                .iter()
                .any(|crate_name| metadata.target().starts_with(crate_name))
            {
                return metadata.level() <= log::LevelFilter::Info;
            }

            metadata.level() <= self.filter
        }

        fn log(&self, record: &log::Record<'_>) {
            use wasm_bindgen::JsValue;

            if !self.enabled(record.metadata()) {
                return;
            }

            let msg = if let (Some(file), Some(line)) = (record.file(), record.line()) {
                let file = if let Some(i) = file.rfind("/src/").or_else(|| file.rfind("\\src\\")) {
                    if let Some(prev_slash) = file[..i].rfind('/').or_else(|| file[..i].rfind('\\'))
                    {
                        &file[prev_slash + 1..]
                    } else {
                        file
                    }
                } else {
                    file
                };
                format!(
                    "%c {} %c [{}] {file}:{line}: {}",
                    record.level(),
                    record.target(),
                    record.args()
                )
            } else {
                format!(
                    "%c {} %c [{}] {}",
                    record.level(),
                    record.target(),
                    record.args()
                )
            };

            let normal_style = JsValue::from_str("color: undefined;");

            let level_style = JsValue::from_str(
                format!(
                    "color: {}; background-color: silver; font-weight: bold;",
                    match record.level() {
                        log::Level::Trace | log::Level::Debug => "blue",
                        log::Level::Info => "green",
                        log::Level::Warn => "darkorange",
                        log::Level::Error => "red",
                    }
                )
                .as_str(),
            );

            let console_log = match record.level() {
                log::Level::Trace | log::Level::Debug => web_sys::console::debug_3,
                log::Level::Info => web_sys::console::info_3,
                log::Level::Warn => web_sys::console::warn_3,
                log::Level::Error => web_sys::console::error_3,
            };

            console_log(&msg.into(), &level_style, &normal_style);
        }

        fn flush(&self) {}
    }

    let logger = Box::new(Logger::new(filter));
    if let Err(e) = log::set_logger(Box::leak(logger)) {
        panic!("Failed to set logger: {e}");
    }
    log::set_max_level(filter);
}
