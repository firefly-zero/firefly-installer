use core::net::{Ipv4Addr, SocketAddrV4};

use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    update_wifi(state);
    if state.wifi_state != WifiState::Connected {
        return;
    }
    update_tcp(state);
    if state.tcp_state != TcpState::Connected {
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

fn update_wifi(state: &mut State) {
    state.wifi_wait = usize::min(state.wifi_wait + 1, 400);
    match state.wifi_state {
        WifiState::NotConnected => {
            state.wifi_wait = 0;
            wifi::connect(&state.ssid, &state.password);
            state.wifi_state = WifiState::Connecting;
        }
        WifiState::Connecting | WifiState::ObtainingIP => {
            let status = wifi::status();
            if state.wifi_wait < 30 {
                return;
            }
            state.wifi_state = match status {
                wifi::Status::Error | wifi::Status::Other => WifiState::Failed,
                wifi::Status::Disconnected => {
                    if state.wifi_wait > 180 {
                        WifiState::Failed
                    } else {
                        WifiState::Connecting
                    }
                }
                wifi::Status::Initializing => WifiState::ObtainingIP,
                wifi::Status::Connected => WifiState::Connected,
            };
        }
        WifiState::Connected => {
            let status = wifi::status();
            if status != wifi::Status::Connected {
                state.wifi_state = WifiState::Failed;
            }
        }
        WifiState::Failed => {}
    }
}

pub fn update_tcp(state: &mut State) {
    state.tcp_wait = usize::min(state.tcp_wait + 1, 400);
    match state.tcp_state {
        TcpState::NotConnected => {
            let ip = Ipv4Addr::new(192, 168, 2, 6);
            let port = 19743;
            let addr = SocketAddrV4::new(ip, port);
            wifi::tcp_connect(addr);
            state.tcp_state = TcpState::Connecting;
        }
        TcpState::Connecting => {
            let status = wifi::tcp_status();
            if state.tcp_wait < 30 {
                return;
            }
            state.tcp_state = match status {
                wifi::TcpStatus::Error => TcpState::Failed,
                wifi::TcpStatus::Established => TcpState::Connected,
                _ => TcpState::Connecting,
            };
        }
        TcpState::Connected => {
            let status = wifi::tcp_status();
            if status != wifi::TcpStatus::Established {
                state.tcp_state = TcpState::Failed;
            }
        }
        TcpState::Failed => {}
    }
}

pub fn render(state: &mut State) {
    let font = state.font.as_font();
    let theme = state.settings.theme;
    let color = theme.primary;

    let wifi_msg = match state.wifi_state {
        WifiState::NotConnected | WifiState::Connecting => "1. Connecting to internet...",
        WifiState::ObtainingIP => "1. Obtaining IP address...",
        WifiState::Connected => "1. Connected to internet.",
        WifiState::Failed => "1. Failed to connect to internet.",
    };
    draw_line(1, wifi_msg, &font, color);
    if state.wifi_state != WifiState::Connected {
        return;
    }

    let tcp_msg = match state.tcp_state {
        TcpState::NotConnected | TcpState::Connecting => "2. Connecting to server...",
        TcpState::Connected => "2. Connected to server.",
        TcpState::Failed => "2. Failed to connect to server.",
    };
    draw_line(2, tcp_msg, &font, color);

    if state.session_id.is_empty() {
        return;
    }
    let id_msg = "3. Session created. ID:";
    draw_line(3, id_msg, &font, color);
    {
        let point = Point::new(38, 12 + 13 * 4);
        draw_text(&state.session_id, &font, point, theme.accent);
    }
}

fn draw_line(i: i32, text: &str, font: &Font, color: Color) {
    let point = Point::new(20, 12 + 13 * i);
    draw_text(text, font, point, color);
}
