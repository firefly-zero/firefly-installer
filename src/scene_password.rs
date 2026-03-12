//! Scene showing the password input prompt.
use crate::*;

pub fn update(state: &mut State) {
    match state.password.update() {
        firefly_keyboard::State::TextChanged => {}
        firefly_keyboard::State::Open => {}
        firefly_keyboard::State::Closed => state.password.open(),
        firefly_keyboard::State::JustClosed => state.transition(Scene::Connection),
        firefly_keyboard::State::JustCancelled => state.transition(Scene::Points),
    }
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let theme = state.settings.theme;

    let text = "enter the password:";
    let point = Point::new(20, 25);
    draw_text(text, &font, point, theme.primary);

    firefly_ui::draw_cursor(1, theme, &font, false, 0);

    let text = state.password.text.as_str();
    let point = Point::new(20, 38);
    draw_text(text, &font, point, theme.accent);

    state.password.render(&font);
}
