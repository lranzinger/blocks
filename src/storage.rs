// Provided by custom.js. Declared as imports, newer Rust versions reject undefined symbols.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn js_get_high_score() -> u32;
    fn js_save_high_score(score: u32);
    fn js_vibrate(milliseconds: u32);
    fn js_is_touch_device() -> u32;
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
