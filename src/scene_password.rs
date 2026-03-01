//! Scene showing the password input prompt.
use crate::*;

pub fn update(state: &mut State) {
    if state.cursor >= state.password.len() - 1 {
        state.password.push('a');
    }
    match state.input.get() {
        Input::Up => {
            let rng = state.cursor..state.cursor + 1;
            let ch = state.password.get(rng.clone()).unwrap();
            let ch = ch.as_bytes()[0];
            let ch = if ch == 0x20 { 0x7e } else { ch - 1 };
            let ch = [ch];
            let ch = unsafe { alloc::str::from_utf8_unchecked(&ch) };
            state.password.replace_range(rng, ch);
        }
        Input::Down => {
            let rng = state.cursor..state.cursor + 1;
            let ch = state.password.get(rng.clone()).unwrap();
            let ch = ch.as_bytes()[0];
            let ch = if ch == 0x7e { 0x20 } else { ch + 1 };
            let ch = [ch];
            let ch = unsafe { alloc::str::from_utf8_unchecked(&ch) };
            state.password.replace_range(rng, ch);
        }
        Input::Left => {
            if state.cursor > 0 {
                state.cursor -= 1
            }
        }
        Input::Right => {
            if state.cursor < 32 {
                state.cursor += 1
            }
        }
        Input::Select => {}
        Input::Back => {}
        Input::None => {}
    }
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
