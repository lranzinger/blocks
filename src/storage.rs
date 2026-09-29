// Provided by custom.js. Declared as imports, newer Rust versions reject undefined symbols.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn js_get_high_score() -> u32;
    fn js_save_high_score(score: u32);
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
