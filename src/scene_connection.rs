use alloc::vec;
use alloc::vec::Vec;
use core::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};
use firefly_rust::*;

use crate::*;

pub fn update(state: &mut State) {
    match state.input.get() {
        Input::Back => {
            // If wifi connection in progress or failed,
            // go back to the password input.
            if state.wifi_state != WifiState::Connected {
                if state.wifi_state != WifiState::Failed {
                    wifi::disconnect();
                }
                state.transition(Scene::Password);
                return;
            }
            // If there is no file download in progress
            // (either already downloaded or no file was sent),
            // going back just exits the app.
            if state.rom_state != RomState::Downloading {
                quit();
                return;
            }
        }
        Input::Select => {
            if state.wifi_state != WifiState::Failed {
                state.wifi_wait = 0;
                wifi::connect(&state.ssid, &state.password.text);
                state.wifi_state = WifiState::Connecting;
            }
            if state.rom_state == RomState::Done {
                quit();
                return;
            }
        }
        _ => {}
    }

    // If an app is already installed, there is nothing else to do.
    if state.rom_state == RomState::Done {
        return;
    }

    update_wifi(state);
    if state.wifi_state != WifiState::Connected {
        return;
    }
    update_tcp(state);
    if state.tcp_state != TcpState::Connected {
        return;
    }

    if state.session_id.is_empty() {
        let session_id = load_session_id();
        wifi::tcp_send(&session_id.to_le_bytes()[..]);
        let mut session_id = alloc::format!("{session_id:08}");
        session_id.insert(4, ' ');
        state.session_id = session_id;
        return;
    }

    update_rom(state);
}

/// Load previously created session ID.
///
/// If none is saved, a new one will be generated.
///
/// Since the ID is preserved between sessions, "device ID" would be a better name.
/// However, we want to avoid confusion with serial number, MAC address,
/// and a ton of other "device IDs".
///
/// We could use the device MAC address instead. However, we want to be able
/// to generate a new ID without changing anything outside of the installer.
fn load_session_id() -> u32 {
    let size = get_file_size("id");
    if size == 4 {
        let mut buf = [0u8; 4];
        load_file("id", &mut buf);
        return u32::from_le_bytes(buf);
    }

    // What's the probability of two users having the same session ID?
    // This is exactly the same statistic problem as in the famous Birthday Paradox.
    // The precise formula uses factorial, which I cannot calculate for 100kk.
    //
    // There is also an approximation based on Taylor series
    // which gives a negligibly small probability (assuming I'm using it right).
    //
    // ```python
    // n = 70_000 # presumed number of devices.
    // d = 100_000_000 # "days" (possible IDs)
    // 1-math.e**(-(n-(n-1))/(d*2))
    // # 4.999999969612645e-09
    // ```
    //
    // https://en.wikipedia.org/wiki/Birthday_problem#Approximations
    let session_id = get_random() % 100_000_000;
    dump_file("id", &session_id.to_le_bytes());
    session_id
}

fn update_wifi(state: &mut State) {
    state.wifi_wait = usize::min(state.wifi_wait + 1, 400);
    match state.wifi_state {
        WifiState::NotConnected => {
            state.wifi_wait = 0;
            wifi::connect(&state.ssid, &state.password.text);
            state.wifi_state = WifiState::Connecting;
        }
        WifiState::Connecting | WifiState::ObtainingIP => {
            let status = wifi::status();
            if state.wifi_wait < 60 {
                return;
            }
            state.wifi_state = match status {
                wifi::Status::Error | wifi::Status::Other => WifiState::Failed,
                wifi::Status::Disconnected => {
                    if state.wifi_wait > 5 * 60 {
                        WifiState::Failed
                    } else {
                        WifiState::Connecting
                    }
                }
                wifi::Status::Initializing => WifiState::ObtainingIP,
                wifi::Status::Connected => {
                    save_password(&state.ssid, &state.password.text);
                    WifiState::Connected
                }
            };
        }
        WifiState::Connected => {
            let status = wifi::status();
            if status != wifi::Status::Connected {
                state.wifi_state = WifiState::Failed;
                if state.tcp_state != TcpState::NotConnected {
                    wifi::tcp_close();
                    state.tcp_state = TcpState::NotConnected;
                }
            }
        }
        WifiState::Failed => {}
    }
}

/// Save the given SSID and password in a data file.
fn save_password(ssid: &str, pass: &str) {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend(ssid.as_bytes());
    buf.push(10); // 10 is '\n'.
    buf.extend(pass.as_bytes());
    dump_file("creds", &buf);
}

fn update_tcp(state: &mut State) {
    state.tcp_wait = usize::min(state.tcp_wait + 1, 400);
    match state.tcp_state {
        TcpState::NotConnected => {
            let addr = load_addr();
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
            // let status = wifi::tcp_status();
            // if status != wifi::TcpStatus::Established {
            //     state.tcp_state = TcpState::Failed;
            //     // TODO: reset ROM download state.
            // }
        }
        TcpState::Failed => {}
    }
}

/// Read server address (IP v4 + port) from the data file.
///
/// Defaults to the IP address of our install.fireflyzero.com server.
///
/// We never write the data file. It can only be created manually
/// by the "advanced" user. We might add it to advanced settings
/// in the future.
fn load_addr() -> SocketAddrV4 {
    let size = get_file_size("addr");
    if size > 0 {
        let mut buf = vec![0; size];
        load_file("addr", &mut buf);
        let buf = buf.trim_ascii();
        let buf = unsafe { alloc::str::from_utf8_unchecked(buf) };
        if let Ok(addr) = SocketAddrV4::from_str(buf) {
            return addr;
        }
    }

    let ip = Ipv4Addr::new(192, 168, 2, 6);
    let port = 19743;
    SocketAddrV4::new(ip, port)
}

fn update_rom(state: &mut State) {
    match state.rom_state {
        RomState::NoResponse => {
            let chunk = wifi::tcp_recv_buf();
            if !chunk.is_empty() {
                state.installer.update(&chunk);
                state.rom_state = RomState::Downloading;
            }
        }
        RomState::Downloading => {
            for _ in 0..20 {
                let chunk = wifi::tcp_recv_buf();
                state.installer.update(&chunk);
                if state.installer.done() {
                    state.rom_state = RomState::Done;
                    break;
                }
                if chunk.is_empty() {
                    break;
                }
            }
        }
        RomState::Done => {}
        RomState::Failed => {}
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

    let rom_msg = match state.rom_state {
        RomState::NoResponse => return,
        RomState::Downloading => "4. Downloading...",
        RomState::Done => "4. Installed.",
        RomState::Failed => "4. Download failed.",
    };
    draw_line(5, rom_msg, &font, color);
}

fn draw_line(i: i32, text: &str, font: &Font, color: Color) {
    let point = Point::new(20, 12 + 13 * i);
    draw_text(text, font, point, color);
}
