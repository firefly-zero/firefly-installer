//! Scene showing the device session ID and waiting for the file.
use core::net::{Ipv4Addr, SocketAddrV4};

use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    if state.cursor == 0 {
        let ip = Ipv4Addr::new(192, 168, 2, 6);
        let port = 19743;
        let addr = SocketAddrV4::new(ip, port);
        wifi::tcp_open(addr);
        state.cursor = 1;
    }

    state.cursor += 1;
    if state.cursor < 30 {
        return;
    }
    let status = wifi::tcp_status();
    if status != wifi::TcpStatus::Established {
        return;
    }

    if state.session_id.is_empty() {
        let session_id = get_random() % 100_000_000;
        wifi::tcp_send(&session_id.to_le_bytes()[..]);
        let mut session_id = alloc::format!("{session_id:08}");
        session_id.insert(4, ' ');
        state.session_id = session_id;
    }
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let text_color = theme.primary;

    let text = if state.session_id.is_empty() {
        "Connecting to the server..."
    } else {
        &state.session_id
    };
    state.rendered_message = true;
    let point = Point::new(40, 40);
    draw_text(text, &font, point, text_color);
}
