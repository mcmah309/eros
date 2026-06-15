fn main() {
    let debug_enabled = std::env::var("CARGO_FEATURE_LOG_DEBUG").is_ok();
    let display_enabled = std::env::var("CARGO_FEATURE_LOG_DISPLAY").is_ok();

    if debug_enabled && display_enabled {
        println!("cargo:warning=eros: Both 'log_debug' and 'log_display' features are active. 'log_debug' takes priority.");
    }
}