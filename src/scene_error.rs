//! Scene showing an error message.
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    if state.input.get() != Input::None {
        state.transition(Scene::Points);
    }
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let text_color = theme.primary;

    let text = "could not connect to Wi-Fi";
    state.rendered_message = true;
    let point = Point::new(40, 40);
    draw_text(text, &font, point, text_color);
}
