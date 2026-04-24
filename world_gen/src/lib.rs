use std::ffi::c_void;

#[repr(C)]
pub struct GameState {
    pub frame_count: u32,
}

// Simple Perlin-like noise (value noise with fade). Replace this function and rebuild to hot-swap.
fn fade(t: f32) -> f32 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + t * (b - a) }
fn hash(i: i32, j: i32) -> f32 {
    let mut n = i.wrapping_mul(374761393) ^ j.wrapping_mul(668265263);
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    ((n & 0x7fffffff) as f32) / 2147483647.0
}

// 2D value-noise based "Perlin-ish" function with range [-1,1]

#[no_mangle]
pub extern "C" fn perlin_noise(x: f32, y: f32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();

    let s = hash(x0, y0) * 2.0 - 1.0;
    let t = hash(x0 + 1, y0) * 2.0 - 1.0;
    let u = hash(x0, y0 + 1) * 2.0 - 1.0;
    let v = hash(x0 + 1, y0 + 1) * 2.0 - 1.0;

    let sx = fade(xf);
    let sy = fade(yf);

    let a = lerp(s, t, sx);
    let b = lerp(u, v, sx);
    let value = lerp(a, b, sy);
    value
}

#[no_mangle]
pub extern "C" fn game_update(state: *mut GameState) {
    if state.is_null() { return; }
    unsafe { (*state).frame_count = (*state).frame_count.wrapping_add(1); }
}
