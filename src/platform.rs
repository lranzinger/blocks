//! Browser functions provided by custom.js

// Declared as imports, newer Rust versions reject undefined symbols.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn js_get_high_score() -> u32;
    fn js_save_high_score(score: u32);
    fn js_vibrate(milliseconds: u32);
    fn js_is_touch_device() -> u32;
    fn js_take_page_hidden() -> u32;
    fn js_now() -> f64;
    fn js_next_touch_time(identifier: u32) -> f64;
    fn js_clear_touch_times();
}

pub fn get_high_score() -> u32 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_get_high_score()
    }

    #[cfg(not(target_arch = "wasm32"))]
    0
}

pub fn update_high_score(score: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_save_high_score(score);
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = score;
}

/// Short haptic feedback on devices that support it
pub fn vibrate(milliseconds: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_vibrate(milliseconds);
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = milliseconds;
}

/// The device is mainly used by touch, like phones and tablets
pub fn is_touch_device() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_is_touch_device() != 0
    }

    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// The page was hidden since the last call, for example by switching tabs or apps
pub fn take_page_hidden() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_take_page_hidden() != 0
    }

    #[cfg(not(target_arch = "wasm32"))]
    false
}

/// Current time in seconds on the clock of the touch times
pub fn now() -> f64 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_now()
    }

    #[cfg(not(target_arch = "wasm32"))]
    macroquad::time::get_time()
}

/// Exact time of the next touch event, in the order the events are replayed
pub fn next_touch_time(identifier: u64) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        let time = unsafe { js_next_touch_time(identifier as u32) };
        (time >= 0.0).then_some(time)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = identifier;
        None
    }
}

/// Drops touch times left over from this frame, so they can't shift into the next one
pub fn clear_touch_times() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_clear_touch_times();
    }
}
