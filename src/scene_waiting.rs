//! Scene showing the device session ID and waiting for the file.
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    if state.session_id.is_empty() {
        let session_id = get_random() % 100_000_000;
        state.session_id = alloc::format!("{session_id}");
    }
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let text_color = theme.primary;

    state.rendered_message = true;
    let point = Point::new(40, 40);
    draw_text(&state.session_id, &font, point, text_color);
}
