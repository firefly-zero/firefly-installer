//! Scene showing "connecting" message when connecting to a Wi-Fi AP.
use firefly_rust::*;

use crate::*;

const MIN_WAIT: usize = 30;
const MAX_WAIT: usize = 180;

pub fn update(state: &mut State) {
    if !state.rendered_message {
        return;
    }
    if state.cursor == 0 {
        wifi::connect(&state.ssid, &state.password);
        state.cursor = 1;
        return;
    }

    // The esp-radio crate doesn't provide the "trying to connect"
    // and "failed to connect" statuses for Wi-Fi, so the only way
    // to ensure that it tried and failed to connect is to wait
    // for long enough and only then check the status.
    state.cursor += 1;
    if state.cursor < MIN_WAIT {
        return;
    }

    let status = wifi::status();
    if status == wifi::Status::Connected {
        state.transition(Scene::Waiting)
    }
    if state.cursor >= MAX_WAIT {
        state.transition(Scene::Error)
    }
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let text_color = theme.primary;

    let text = "Connecting to Wi-Fi...";
    state.rendered_message = true;
    let point = Point::new(40, 40);
    draw_text(text, &font, point, text_color);
}
