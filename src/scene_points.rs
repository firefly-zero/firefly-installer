//! Scene showing the list of WiFi access points.
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    if !state.rendered_message {
        return;
    }
    let points = wifi::scan();
    state.points = Some(points);
}

pub fn render(state: &mut State) {
    state.rendered_message = true;
    let font = state.font.as_font();
    let text_color = Color::Black;

    let Some(points) = &state.points else {
        let text = "scanning...";
        let point = Point::new(40, 40);
        draw_text(text, &font, point, text_color);
        return;
    };

    for (ssid, i) in points.iter().zip(1..) {
        let point = Point::new(10, 10 + 10 * i);
        draw_text(ssid, &font, point, text_color);
    }
}
