#![no_std]
#![no_main]
extern crate alloc;

mod bindings;
mod scene_points;
mod state;
mod wifi;

use firefly_rust as ff;
use state::*;

#[unsafe(no_mangle)]
extern "C" fn boot() {
    load_state();
}

#[unsafe(no_mangle)]
extern "C" fn before_exit() {
    wifi::disconnect();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    scene_points::update(state);
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    ff::clear_screen(ff::Color::White);
    let state = get_state();
    scene_points::render(state);
}
