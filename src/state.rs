use crate::*;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::OnceCell;
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
    Failed,
}

pub struct State {
    pub font: FileBuf,
    pub settings: Settings,
    pub points: Vec<String>,
    pub rendered_message: bool,
    pub ssid: String,
    pub password: String,
    pub session_id: String,
    pub scene: Scene,
    pub cursor: usize,
    pub input: InputManager,
    pub installer: Installer,

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
        self.scene = scene;
    }
}

pub fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe { STATE.get_mut() }.unwrap()
}

pub fn load_state() {
    let font = load_file_buf("ascii").unwrap();
    let password = option_env!("WIFI_PASSWORD").unwrap_or_default();
    let password = String::from(password);
    let state = State {
        font,
        settings: get_settings(get_me()),
        points: Vec::new(),
        rendered_message: false,
        ssid: String::new(),
        password,
        session_id: String::new(),
        scene: Scene::Points,
        cursor: 0,
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
