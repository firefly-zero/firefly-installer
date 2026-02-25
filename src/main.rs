#![no_std]
#![no_main]
extern crate alloc;

mod bindings;
mod input;
mod scene_password;
mod scene_points;
mod state;
mod wifi;

use firefly_rust as ff;
use input::*;
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
    state.input.update();
    match state.scene {
        Scene::Points => scene_points::update(state),
        Scene::Password => scene_password::update(state),
    }
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    ff::clear_screen(ff::Color::White);
    let state = get_state();
    match state.scene {
        Scene::Points => scene_points::render(state),
        Scene::Password => scene_password::render(state),
    }
}
