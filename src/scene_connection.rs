use alloc::vec;
use alloc::vec::Vec;
use core::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};
use firefly_rust::*;
use firefly_sudo::sudo;

use crate::*;

pub fn update(state: &mut State) {
    let state_changed = handle_input(state);
    if state_changed {
        return;
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
        let mut session_id = alloc::format!("{session_id:08}");

        let header = alloc::format!(
            "GET /download/{session_id} HTTP/1.1\r\nHost: install.fireflyzero.com\r\n\r\n"
        );
        wifi::tcp_send(header.as_bytes());

        session_id.insert(4, ' ');
        state.session_id = session_id;
        return;
    }

    update_rom(state);
}

fn handle_input(state: &mut State) -> bool {
    match state.input.get() {
        Input::Back => {
            // If wifi connection in progress or failed,
            // go back to the password input.
            if state.wifi_state != WifiState::Connected {
                if state.wifi_state != WifiState::Failed {
                    wifi::disconnect();
                }
                state.wifi_state = WifiState::NotConnected;
                state.transition(Scene::Password);
                return true;
            }
            // If there is no file download in progress
            // (either already downloaded or no file was sent),
            // going back just exits the app.
            if state.rom_state != RomState::Downloading {
                quit();
                return true;
            }
        }
        Input::Select => {
            let done = state.rom_state == RomState::Done;
            if done {
                match state.cursor {
                    0 => restart(),
                    1 => {
                        let (author_id, app_id) = state.installer.get_id();
                        if state.installer.has_manual {
                            let target = alloc::format!("{author_id}.{app_id}");
                            sudo::dump_file("data/sys/manuals/etc/target", target.as_bytes());
                            sudo::run_app("sys", "manuals");
                        } else {
                            sudo::run_app(author_id, app_id);
                        }
                    }
                    2 => quit(),
                    _ => {}
                }
                return true;
            }
            if state.wifi_state == WifiState::Failed {
                state.wifi_wait = 0;
                wifi::connect(&state.ssid, &state.password.text);
                state.wifi_state = WifiState::Connecting;
                return true;
            }
        }
        Input::Up => {
            if state.cursor > 0 {
                state.cursor -= 1;
            }
        }
        Input::Down => {
            if state.cursor < 2 {
                state.cursor += 1;
            }
        }
        Input::Left => state.cursor = 0,
        Input::Right => state.cursor = 2,
        Input::None => {}
    }
    false
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
                    save_creds(&state.ssid, &state.password.text);
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
///
/// It will be loaded on the next launch by `load_creds`.
fn save_creds(ssid: &str, pass: &str) {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend(ssid.as_bytes());
    buf.push(b'\n');
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
            let status = wifi::tcp_status();
            if status == wifi::TcpStatus::Error {
                state.tcp_state = TcpState::Failed;
                if state.rom_state == RomState::NoResponse {
                    state.session_id.clear();
                }
            }
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

    let ip = Ipv4Addr::new(116, 203, 135, 82); // install.fireflyzero.com
    SocketAddrV4::new(ip, 80)
}

fn update_rom(state: &mut State) {
    match state.rom_state {
        RomState::NoResponse => {
            let chunk = wifi::tcp_recv_buf();
            if !chunk.is_empty() {
                let res = state.installer.update(&chunk);
                if let Err(err) = res {
                    state.rom_state = RomState::Failed(err);
                    return;
                }
                state.rom_state = RomState::Downloading;
            }
        }
        RomState::Downloading => {
            let chunk = download_chunk();
            let res = state.installer.update(&chunk);
            if let Err(err) = res {
                state.rom_state = RomState::Failed(err);
                return;
            }
            if state.installer.done() {
                let res = state.installer.finalize();
                state.rom_state = if let Err(err) = res {
                    RomState::Failed(err)
                } else {
                    RomState::Done
                };
            }
        }
        RomState::Done => {}
        RomState::Failed(_) => {}
    }
}

/// Fetch a chunk of response body.
fn download_chunk() -> Vec<u8> {
    const N_PULLS: usize = 40;
    const PULL_SIZE: usize = 80;

    let mut chunk = vec![0; N_PULLS * PULL_SIZE];
    let mut chunk_size = 0;
    for _ in 0..N_PULLS {
        if chunk.is_empty() {
            break;
        }
        let n = wifi::tcp_recv(&mut chunk[chunk_size..chunk_size + PULL_SIZE]);
        chunk_size += n;
        if n == 0 {
            break;
        }
    }
    chunk.truncate(chunk_size);
    chunk
}

pub fn render(state: &mut State) {
    draw_messages(state);
    draw_buttons(state);
}

fn draw_messages(state: &mut State) {
    let font = &state.font;
    let theme = state.settings.theme;
    let color = theme.primary;

    let wifi_msg = match state.wifi_state {
        WifiState::NotConnected | WifiState::Connecting => "1. Connecting to internet...",
        WifiState::ObtainingIP => "1. Obtaining IP address...",
        WifiState::Connected => "1. Connected to internet.",
        WifiState::Failed => "1. Failed to connect to internet.",
    };
    draw_text_line(1, wifi_msg, font, color);
    if state.wifi_state != WifiState::Connected {
        return;
    }

    let tcp_msg = match state.tcp_state {
        TcpState::NotConnected | TcpState::Connecting => "2. Connecting to server...",
        TcpState::Connected => "2. Connected to server.",
        TcpState::Failed => "2. Failed to connect to server.",
    };
    draw_text_line(2, tcp_msg, font, color);

    if state.session_id.is_empty() {
        return;
    }
    let id_msg = "3. Session created. ID:";
    draw_text_line(3, id_msg, font, color);
    {
        let point = Point::new(38, 12 + 13 * 4);
        draw_text(&state.session_id, font, point, theme.accent);
    }

    let rom_msg = match state.rom_state {
        RomState::NoResponse => return,
        RomState::Downloading => "4. Downloading...",
        RomState::Done => "4. Installed.",
        RomState::Failed(err) => {
            let point = Point::new(38, 12 + 13 * 6);
            draw_text(err, font, point, theme.accent);
            "4. Download failed:"
        }
    };
    draw_text_line(5, rom_msg, font, color);

    // Progress bar.
    if state.rom_state == RomState::Downloading {
        let point = Point::new(20, 12 + 13 * 6 - 6);
        let style = Style::outlined(theme.primary, 1);
        let full_width = 140;
        draw_rect(point, Size::new(full_width, 6), style);

        let style = Style::solid(theme.accent);
        let ratio = state.installer.flushed();
        let part_width = (full_width as f32 * ratio) as i32;
        draw_rect(point, Size::new(part_width, 6), style);

        let style = Style::outlined(theme.primary, 1);
        let ratio = state.installer.downloaded();
        let part_width = (full_width as f32 * ratio) as i32;
        draw_rect(point, Size::new(part_width, 6), style);
    }
}

fn draw_buttons(state: &mut State) {
    let done = state.rom_state == RomState::Done;
    if !done {
        return;
    }

    let font = &state.font;
    let theme = state.settings.theme;
    let color = theme.primary;

    let pressed = state.input.pressed();
    firefly_ui::draw_cursor(6 + state.cursor as u32, theme, font, pressed, 0);

    draw_line(
        Point::new(12, 93),
        Point::new(227, 93),
        LineStyle::new(theme.primary, 1),
    );

    {
        let pressed = pressed && state.cursor == 0;
        draw_button_text(7, "install another app", font, color, pressed);
    }
    {
        let pressed = pressed && state.cursor == 1;
        let msg = if state.installer.has_manual {
            "read app manual"
        } else {
            "launch installed app"
        };
        draw_button_text(8, msg, font, color, pressed);
    }
    {
        let pressed = pressed && state.cursor == 2;
        draw_button_text(9, "exit", font, color, pressed);
    }
}

fn draw_text_line(i: i32, text: &str, font: &FontBuf, color: Color) {
    let point = Point::new(20, 12 + 13 * i);
    draw_text(text, font, point, color);
}

fn draw_button_text(i: i32, text: &str, font: &FontBuf, color: Color, pressed: bool) {
    let mut point = Point::new(20, 12 + 13 * i);
    if pressed {
        point.x += 1;
        point.y += 1;
    }
    draw_text(text, font, point, color);
}
