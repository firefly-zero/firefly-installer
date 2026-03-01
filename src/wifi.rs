use crate::*;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

/// Connect to a Wi-Fi access point.
pub fn connect(ssid: &str, pass: &str) -> Result<(), ()> {
    let err = unsafe {
        bindings::connect(
            ssid.as_ptr() as u32,
            ssid.len() as u32,
            pass.as_ptr() as u32,
            pass.len() as u32,
        )
    };
    if err == 0 { Ok(()) } else { Err(()) }
}

/// Close Wi-Fi connection.
pub fn disconnect() {
    unsafe {
        bindings::disconnect();
    }
}

/// List SSIDs of the top few Wi-Fi access points.
pub fn scan() -> Vec<String> {
    let mut buf: Vec<u8> = vec![0; 200];
    let len = unsafe { bindings::scan(buf.as_mut_ptr() as u32, buf.len() as u32) };
    let mut raw = &buf[..len as usize];

    let mut points: Vec<String> = Vec::new();
    while !raw.is_empty() {
        let len = raw[0] as usize;
        let Some(raw_name) = raw.get(1..=len) else {
            break;
        };
        let name = core::str::from_utf8(raw_name).unwrap_or("");
        raw = raw.get((len + 1)..).unwrap_or(&[]);
        let name = String::from(name);
        points.push(name);
    }
    points
}
