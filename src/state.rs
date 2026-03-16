use crate::*;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::OnceCell;
use firefly_keyboard::Keyboard;
use firefly_rust::*;

static mut STATE: OnceCell<State> = OnceCell::new();

#[derive(Copy, Clone)]
pub enum Scene {
    Points,
    Password,
    Connection,
}

#[derive(PartialEq)]
pub enum WifiState {
    NotConnected,
    Connecting,
    ObtainingIP,
    Connected,
    Failed,
}

#[derive(PartialEq)]
pub enum TcpState {
    NotConnected,
    Connecting,
    Connected,
    Failed,
}

#[derive(PartialEq)]
pub enum RomState {
    NoResponse,
    Downloading,
    Done,
    Failed(&'static str),
}

pub struct State {
    pub font: FileBuf,
    pub settings: Settings,
    pub points: Vec<String>,
    pub rendered_message: bool,
    pub session_id: String,
    pub scene: Scene,
    pub cursor: usize,
    pub input: InputManager,
    pub installer: Installer,

    pub ssid: String,
    pub password: Keyboard,
    pub show_password: u8,
    pub saved: Option<(String, String)>,

    pub wifi_state: WifiState,
    pub wifi_wait: usize,
    pub tcp_state: TcpState,
    pub tcp_wait: usize,
    pub rom_state: RomState,
}

impl State {
    pub fn transition(&mut self, scene: Scene) {
        self.rendered_message = false;
        self.cursor = 0;
        self.wifi_wait = 0;
        self.tcp_wait = 0;
        self.show_password = 0;
        self.scene = scene;
    }
}

pub fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe { STATE.get_mut() }.unwrap()
}

pub fn load_state() {
    let font = load_file_buf("ascii").unwrap();

    // If there is already saved SSID+password for an AP,
    // go to the connection directly. If that doesn't work,
    // the user can always go back to the previous screens.
    let saved = load_creds();
    let mut ssid = String::new();
    let mut password = Keyboard::default();
    let mut scene = Scene::Points;
    if let Some((saved_ssid, saved_pass)) = saved.as_ref() {
        ssid = saved_ssid.clone();
        password.text = saved_pass.clone();
        scene = Scene::Connection;
    }

    let state = State {
        font,
        settings: get_settings(get_me()),
        points: Vec::new(),
        rendered_message: false,
        ssid,
        password,
        saved,
        session_id: String::new(),
        scene,
        cursor: 0,
        show_password: 0,
        input: InputManager::new(),
        installer: Installer::new(),

        wifi_state: WifiState::NotConnected,
        wifi_wait: 0,
        tcp_state: TcpState::NotConnected,
        tcp_wait: 0,
        rom_state: RomState::NoResponse,
    };
    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
}

/// Load the stored SSID+password, if any, for the latest wifi AP.
///
/// The password might be an empty string if the network is public.
///
/// The credentials are stored in a data file.
/// Only the latest password for the latest used AP is stored.
fn load_creds() -> Option<(String, String)> {
    let size = get_file_size("creds");
    let mut buf = vec![0u8; size];
    load_file("creds", &mut buf[..]);
    let raw = buf.trim_ascii();
    let raw = alloc::str::from_utf8(&raw[1..]).ok()?;
    let (ssid, pass) = split_by(raw, '\n')?;
    let creds = (String::from(ssid), String::from(pass));
    Some(creds)
}

/// Split the string once at the given character.
fn split_by(input: &str, sep: char) -> Option<(&str, &str)> {
    let mut split_at = None;
    let sep: u8 = sep.try_into().unwrap();
    for (i, ch) in input.bytes().enumerate() {
        if ch == sep {
            split_at = Some(i);
            break;
        }
    }
    let split_at = split_at?;
    Some(input.split_at(split_at))
}
