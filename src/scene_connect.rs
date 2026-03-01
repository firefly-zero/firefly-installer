//! Scene showing "connecting" message when connecting to a Wi-Fi AP.
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    if !state.rendered_message {
        return;
    }
    if state.connected {
        if state.input.get() != Input::None {
            state.transition(Scene::Waiting);
        }
        return;
    }
    let res = wifi::connect(&state.ssid, &state.password);
    if res.is_ok() {
        state.connected = true;
    } else {
        state.transition(Scene::Error);
    }
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let text_color = theme.primary;

    let text = if state.rendered_message {
        "Connecting to Wi-Fi..."
    } else {
        "Connected!"
    };
    state.rendered_message = true;
    let point = Point::new(40, 40);
    draw_text(text, &font, point, text_color);
}
