#![no_std]
#![no_main]
extern crate alloc;

mod bindings;
mod scene_connect;
mod scene_error;
mod scene_password;
mod scene_points;
mod scene_waiting;
mod state;
mod wifi;

use firefly_rust::*;
use firefly_ui::{Input, InputManager};
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
        Scene::Connect => scene_connect::update(state),
        Scene::Error => scene_error::update(state),
        Scene::Waiting => scene_waiting::update(state),
    }
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    firefly_ui::draw_bg(state.settings.theme);
    match state.scene {
        Scene::Points => scene_points::render(state),
        Scene::Password => scene_password::render(state),
        Scene::Connect => scene_connect::render(state),
        Scene::Error => scene_error::render(state),
        Scene::Waiting => scene_waiting::render(state),
    }
}
