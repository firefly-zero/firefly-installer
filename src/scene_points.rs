//! Scene showing the list of WiFi access points.
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    if !state.rendered_message {
        return;
    }
    let points = match state.points.as_ref() {
        Some(points) => points,
        None => {
            let points = wifi::scan();
            state.points = Some(points);
            state.points.as_ref().unwrap()
        }
    };

    if points.is_empty() {
        return;
    }
    match state.input.get() {
        Input::Up => {
            if state.cursor > 0 {
                state.cursor -= 1;
            }
        }
        Input::Down => {
            if state.cursor < points.len() - 1 {
                state.cursor += 1;
            }
        }
        Input::Select => {
            state.ssid = points[state.cursor].clone();
            state.transition(Scene::Password);
        }
        Input::Back => quit(),
        _ => {}
    }
}

pub fn render(state: &mut State) {
    state.rendered_message = true;
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let text_color = theme.primary;

    let Some(points) = &state.points else {
        let text = "Scanning...";
        let point = Point::new(40, 40);
        draw_text(text, &font, point, text_color);
        return;
    };
    if points.is_empty() {
        let text = "No access points found";
        let point = Point::new(40, 40);
        draw_text(text, &font, point, text_color);
        return;
    }

    firefly_ui::draw_cursor(state.cursor as u32, theme, &font, state.input.pressed(), 0);
    for (ssid, i) in points.iter().zip(1..) {
        let point = Point::new(10, 10 + 10 * i);
        draw_text(ssid, &font, point, text_color);
    }
}
