//! Scene showing "connecting" message when connecting to a Wi-Fi AP.
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    if !state.rendered_message {
        return;
    }
    if state.wifi_status.is_none() {
        wifi::connect(&state.ssid, &state.password);
        state.wifi_status = Some(wifi::Status::Disconnected);
        return;
    }
    let status = wifi::status();
    state.wifi_status = Some(status);
    match status {
        wifi::Status::Connected => {
            if state.input.get() != Input::None {
                state.transition(Scene::Waiting)
            }
        }
        // TODO: timeout
        wifi::Status::Disconnected => {}
        wifi::Status::Error | wifi::Status::Other => state.transition(Scene::Error),
    }
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let text_color = theme.primary;

    let text = if state.wifi_status == Some(wifi::Status::Connected) {
        "Connected!"
    } else {
        "Connecting to Wi-Fi..."
    };
    state.rendered_message = true;
    let point = Point::new(40, 40);
    draw_text(text, &font, point, text_color);
}
