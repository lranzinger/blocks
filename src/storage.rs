// Provided by custom.js. Declared as imports, newer Rust versions reject undefined symbols.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn js_get_high_score() -> u32;
    fn js_save_high_score(score: u32);
    fn js_now() -> f64;
    fn js_next_touch_time(identifier: u32) -> f64;
    fn js_clear_touch_times();
}

#[cfg(not(target_arch = "wasm32"))]
fn js_get_high_score() -> u32 {
    // Mock implementation for local builds
    println!("Using local mock for js_get_high_score");
    0
}

#[cfg(not(target_arch = "wasm32"))]
fn js_save_high_score(score: u32) {
    // Mock implementation for local builds
    println!("Using local mock for js_save_high_score: {}", score);
}

pub fn get_high_score() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_get_high_score()
    }

    #[cfg(not(target_arch = "wasm32"))]
    js_get_high_score()
}

pub fn update_high_score(score: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_save_high_score(score);
    }

    #[cfg(not(target_arch = "wasm32"))]
    js_save_high_score(score);
}

/// Time of the next touch event on the game clock, in the order the events are replayed.
/// The browser knows when the finger actually moved, the game only once per frame.
pub fn next_touch_time(identifier: u64) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        let time = unsafe { js_next_touch_time(identifier as u32) };
        // Convert from the browser clock to the game clock
        (time >= 0.0).then(|| time - unsafe { js_now() } + macroquad::time::get_time())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = identifier;
        None
    }
}

/// Drops touch times left over, so they can't shift to other events
pub fn clear_touch_times() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_clear_touch_times();
    }
}
