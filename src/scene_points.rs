//! Scene showing the list of WiFi access points.
use alloc::{string::String, vec::Vec};
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    if !state.rendered_message {
        return;
    }

    state.wifi_wait += 1;
    if state.wifi_wait <= 5 {
        let new_points = wifi::scan();
        match state.points.as_mut() {
            Some(old_points) => merge_points(old_points, new_points),
            None => state.points = Some(new_points),
        }
    }
    let points = state.points.as_ref().unwrap();

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

fn merge_points(old_points: &mut Vec<String>, new_points: Vec<String>) {
    for point in new_points {
        if !old_points.contains(&point) {
            old_points.push(point);
        }
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
        let point = Point::new(20, 12 + 13 * i);
        draw_text(ssid, &font, point, text_color);
    }
}
