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
    Connect,
    Error,
    Waiting,
}

pub struct State {
    pub font: FileBuf,
    pub settings: Settings,
    pub points: Option<Vec<String>>,
    pub rendered_message: bool,
    pub ssid: String,
    pub password: String,
    pub session_id: String,
    pub scene: Scene,
    pub cursor: usize,
    pub input: InputManager,
}

impl State {
    pub fn transition(&mut self, scene: Scene) {
        self.rendered_message = false;
        self.cursor = 0;
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
        points: None,
        rendered_message: false,
        ssid: String::new(),
        password,
        session_id: String::new(),
        scene: Scene::Points,
        cursor: 0,
        input: InputManager::new(),
    };
    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
}
