//! Scene showing the password input prompt.
use crate::*;

pub fn update(state: &mut State) {
    state.cursor = state.cursor.wrapping_add(1);
    state.show_password = state.show_password.saturating_sub(1);
    let old_len = state.password.text.len();
    match state.password.update() {
        firefly_keyboard::State::TextChanged => {
            if state.password.text.len() < old_len {
                state.show_password = 60;
            }
        }
        firefly_keyboard::State::Open => {}
        firefly_keyboard::State::Closed => state.password.open(),
        firefly_keyboard::State::JustClosed => state.transition(Scene::Connection),
        firefly_keyboard::State::JustCancelled => state.transition(Scene::Points),
    }
}

pub fn render(state: &mut State) {
    let font = &state.font;
    let theme = state.settings.theme;

    let text = "enter the password:";
    let point = Point::new(20, 25);
    draw_text(text, font, point, theme.primary);

    firefly_ui::draw_cursor(1, theme, font, false, 0);

    let text = state.password.text.as_str();
    if state.show_password != 0 {
        let point = Point::new(20, 38);
        draw_text(text, font, point, theme.accent);
    } else {
        for i in 0..text.len() {
            draw_circle(
                Point::new(21 + i as i32 * i32::from(font.char_width()), 34),
                i32::from(font.char_width() - 2),
                Style::solid(theme.accent),
            );
        }
    }

    if (state.cursor / 40).is_multiple_of(2) {
        draw_rect(
            Point::new(
                20 + font.line_width_ascii(text) as i32,
                41 - i32::from(font.char_height()),
            ),
            Size::new(font.char_width(), font.char_height() - 1),
            Style::solid(theme.accent),
        );
    }

    state.password.render(font);
}
