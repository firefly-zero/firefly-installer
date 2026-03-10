#![no_std]
#![no_main]
extern crate alloc;

mod bindings;
mod installer;
mod scene_connection;
mod scene_password;
mod scene_points;
mod state;
mod wifi;

use firefly_rust::*;
use firefly_ui::{Input, InputManager};
use installer::*;
use state::*;

#[unsafe(no_mangle)]
extern "C" fn boot() {
    load_state();
}

#[unsafe(no_mangle)]
extern "C" fn before_exit() {
    wifi::tcp_close();
    wifi::disconnect();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    state.input.update();
    match state.scene {
        Scene::Points => scene_points::update(state),
        Scene::Password => scene_password::update(state),
        Scene::Connection => scene_connection::update(state),
    }
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    firefly_ui::draw_bg(state.settings.theme);
    match state.scene {
        Scene::Points => scene_points::render(state),
        Scene::Password => scene_password::render(state),
        Scene::Connection => scene_connection::render(state),
    }
}
