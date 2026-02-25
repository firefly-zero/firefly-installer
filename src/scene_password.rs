//! Scene showing the password input prompt.
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    // ...
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let text_color = state.settings.theme.primary;

    let mut text = state.password.as_str();
    if text.is_empty() {
        text = "enter the password";
    }
    let point = Point::new(40, 40);
    draw_text(text, &font, point, text_color);
}
