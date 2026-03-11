//! Scene showing the list of WiFi access points.
use crate::*;
use alloc::{string::String, vec::Vec};
use firefly_rust::*;

pub fn update(state: &mut State) {
    if !state.rendered_message {
        return;
    }

    state.wifi_wait += 1;
    if state.wifi_wait <= 5 {
        scan_points(state);
    }

    if state.points.is_empty() {
        return;
    }
    match state.input.get() {
        Input::Up => {
            if state.cursor > 0 {
                state.cursor -= 1;
            }
        }
        Input::Down => {
            if state.cursor < state.points.len() - 1 {
                state.cursor += 1;
            }
        }
        Input::Select => {
            state.ssid = state.points[state.cursor].clone();
            state.transition(Scene::Password);
        }
        Input::Back => quit(),
        _ => {}
    }
}

/// Scan for more wifi Access Points and add them to the list.
fn scan_points(state: &mut State) {
    let new_points = wifi::scan();
    merge_points(&mut state.points, new_points);
    // Sort APs after the cursor.
    // We don't touch the AP under the cursor to avoid
    // moving the AP just before the user tries to click it.
    // We also don't touch the APs before the cursor
    // so that the APs that the user already scrolled through
    // are not mixed with the APs that the user is yet to see.
    if state.points.len() > state.cursor + 1 {
        bubble_sort(&mut state.points[state.cursor + 1..]);
    }
}

/// Add new APs to the end of the APs list, avoiding duplicates.
fn merge_points(old_points: &mut Vec<String>, new_points: Vec<String>) {
    for point in new_points {
        if !old_points.contains(&point) {
            old_points.push(point);
        }
    }
}

/// Good old bubble sort. Slower but much smaller than the built-in sort function.
pub fn bubble_sort(items: &mut [String]) {
    let len = items.len();
    if len <= 1 {
        return;
    }
    let mut sorted = false;
    while !sorted {
        sorted = true;
        for i in 0..len - 1 {
            if ascii_gt(&items[i], &items[i + 1]) {
                items.swap(i, i + 1);
                sorted = false;
            }
        }
    }
}

/// Case-insensitive comparison of two ASCII strings.
pub fn ascii_gt(s1: &str, s2: &str) -> bool {
    for (c1, c2) in s1.as_bytes().iter().zip(s2.as_bytes()) {
        let c1 = c1.to_ascii_lowercase();
        let c2 = c2.to_ascii_lowercase();
        if c1 != c2 {
            return c1 > c2;
        }
    }
    s1.len() > s2.len()
}

pub fn render(state: &mut State) {
    state.rendered_message = true;
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let text_color = theme.primary;

    if state.points.is_empty() {
        let text = if state.wifi_wait <= 5 {
            "Scanning..."
        } else {
            "No access points found"
        };
        let point = Point::new(40, 40);
        draw_text(text, &font, point, text_color);
        return;
    }

    firefly_ui::draw_cursor(state.cursor as u32, theme, &font, state.input.pressed(), 0);
    for (ssid, i) in state.points.iter().zip(1..) {
        let point = Point::new(20, 12 + 13 * i);
        draw_text(ssid, &font, point, text_color);
    }
}
